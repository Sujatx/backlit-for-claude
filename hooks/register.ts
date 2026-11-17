import type { EngineInterface, Register } from 'claude-code'

type State = 'working' | 'attention' | 'done' | 'idle'

const IDLE_AFTER_DONE_MS = 30_000
// Notifications that mean "Claude is waiting for you"; others (idle_prompt, auth_success) do not.
const NEEDS_YOU = ['permission_prompt', 'elicitation_dialog']

let current: State | undefined
let idleTimer: { cancel(): void } | undefined
// One kbl call at a time, so rapid events cannot land out of order.
let queue: Promise<unknown> = Promise.resolve()

function set($: EngineInterface, state: State): Promise<unknown> {
  if (state === current) return queue
  current = state
  idleTimer?.cancel()
  idleTimer = state === 'done' ? $.clock.after(IDLE_AFTER_DONE_MS, () => void set($, 'idle')) : undefined
  queue = queue.then(() =>
    $.process.run([`${$.plugin.root}/bin/kbl.exe`, 'state', state], { timeoutMs: 5000 }).catch(() => undefined),
  )
  return queue
}

// The ask is over once a tool ran, failed, was denied, or the model went on to its next call.
function resume($: EngineInterface) {
  if (current === 'attention') void set($, 'working')
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    const started = await next(e)
    void set($, 'idle')
    return started
  })

  on('turn.start', async ($, e, next) => {
    if (e.agentId === undefined) void set($, 'working')
    return next(e)
  })

  on('classic.PermissionRequest', async ($, e, next) => {
    void set($, 'attention')
    return next(e)
  })
  on('classic.Notification', async ($, e, next) => {
    if (NEEDS_YOU.includes(e.notification_type)) void set($, 'attention')
    return next(e)
  })
  on('classic.PostToolUse', async ($, e, next) => {
    resume($)
    return next(e)
  })
  on('classic.PostToolUseFailure', async ($, e, next) => {
    resume($)
    return next(e)
  })
  on('classic.PermissionDenied', async ($, e, next) => {
    resume($)
    return next(e)
  })
  // A refused ask runs no tool, so the model's next call is what shows the ask is over.
  on('tool.call', async ($, e, next) => {
    resume($)
    return next(e)
  })

  on('turn.complete', async ($, e, next) => {
    if (e.agentId === undefined) {
      void set($, e.reason === 'answer' ? 'done' : e.reason === 'aborted' ? 'idle' : 'attention')
    }
    return next(e)
  })

  on('session.end', async ($, e, next) => {
    await set($, 'idle')
    return next(e)
  })
}
