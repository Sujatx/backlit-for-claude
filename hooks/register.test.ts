import { expect, test } from 'claude-code/testing'

test('the light follows the turn and ignores notifications that do not need the person', async ($, on) => {
  const states: string[] = []
  // Nothing sits beneath the mod: this hook stands for the engine's process runner.
  on('process.run', (_, e) => {
    states.push(e.argv[2] ?? '')
    return { value: { exitCode: 0, stdout: '', stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('turn.start', () => ({ turnId: 't1' }))
  on('tool.call', () => ({ result: {} }))
  on('classic.Notification', () => ({}))

  await $.session.start({ cwd: '/', surface: 'desktop', isInteractive: true })
  await $.turn.start({ turnId: 't1', text: 'hi' })
  await $.classic.Notification({ message: 'signed in', notification_type: 'auth_success' })
  await $.classic.Notification({ message: 'needs approval', notification_type: 'permission_prompt' })
  // The ask was refused: the model's next call shows it is over.
  await $.tool.call({ tool: 'Bash', command: 'ls' })

  expect(states).toEqual(['idle', 'working', 'attention', 'working'])
})
