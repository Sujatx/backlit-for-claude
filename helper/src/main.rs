//! ASUS TUF keyboard lighting via \.\ATKACPI (protocol from G-Helper's AsusACPI.cs).
//! Usage: kbl state <working|attention|done|idle>
//!        kbl brightness <0-3>
//!        kbl rgb <mode> <r> <g> <b> <speed>
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING};
use windows_sys::Win32::System::IO::DeviceIoControl;

const IOCTL: u32 = 0x0022240C;
const DEVS: u32 = 0x5356_4544; // "DEVS" little-endian
const TUF_KB_BRIGHTNESS: u32 = 0x0005_0021;
const TUF_KB: u32 = 0x0010_0056;
const TUF_KB2: u32 = 0x0010_005A;

struct Acpi(isize);

impl Acpi {
    fn open() -> Result<Acpi, String> {
        let name: Vec<u16> = r"\\.\ATKACPI".encode_utf16().chain(Some(0)).collect();
        let h = unsafe {
            CreateFileW(name.as_ptr(), GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(), OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, null_mut())
        };
        if h == INVALID_HANDLE_VALUE { Err(format!("cannot open ATKACPI device (win32 error {})", unsafe { windows_sys::Win32::Foundation::GetLastError() })) } else { Ok(Acpi(h as isize)) }
    }

    /// DEVS call: [DEVS][len][device_id][payload]; returns the first output dword (1 = success).
    fn devs(&self, device: u32, payload: &[u8]) -> i32 {
        let mut args = device.to_le_bytes().to_vec();
        args.extend_from_slice(payload);
        let mut inp = DEVS.to_le_bytes().to_vec();
        inp.extend_from_slice(&(args.len() as u32).to_le_bytes());
        inp.extend_from_slice(&args);
        let mut out = [0u8; 16];
        let mut ret = 0u32;
        unsafe {
            DeviceIoControl(self.0 as _, IOCTL, inp.as_ptr().cast(), inp.len() as u32,
                out.as_mut_ptr().cast(), 16, &mut ret, null_mut());
        }
        i32::from_le_bytes(out[..4].try_into().unwrap())
    }

    fn brightness(&self, level: u8) -> i32 {
        self.devs(TUF_KB_BRIGHTNESS, &(0x80u32 | (level as u32 & 0x7F)).to_le_bytes())
    }

    fn rgb(&self, mode: u8, r: u8, g: u8, b: u8, speed: u8) -> i32 {
        let mut p = [0xB4, mode, r, g, b, speed];
        let res = self.devs(TUF_KB, &p);
        if res != 1 {
            p[0] = 0xB3;
            self.devs(TUF_KB2, &p);
            p[0] = 0xB4;
            return self.devs(TUF_KB2, &p);
        }
        res
    }
}

impl Drop for Acpi {
    fn drop(&mut self) { unsafe { CloseHandle(self.0 as _) }; }
}

fn num<T: std::str::FromStr>(a: &[String], i: usize) -> Result<T, String> {
    a.get(i).ok_or("missing argument")?.parse().map_err(|_| format!("bad argument {}", a[i]))
}

fn run(a: &[String]) -> Result<(), String> {
    let acpi = Acpi::open()?;
    let rgb = match a.first().map(String::as_str) {
        Some("state") => match a.get(1).map(String::as_str) {
            // [mode, r, g, b, speed]; mode 0 static, 1 breathe, 2 colour cycle.
            // Brightness is never touched, so the Fn-key level stays the person's own.
            Some("working") => [1, 255, 100, 0, 1],
            Some("attention") => [0, 255, 0, 0, 0],
            Some("done") => [0, 0, 255, 0, 0],
            Some("idle") => [2, 255, 255, 255, 1],
            _ => return Err("state: working|attention|done|idle".into()),
        },
        Some("brightness") => {
            println!("brightness={}", acpi.brightness(num(a, 1)?));
            return Ok(());
        }
        Some("rgb") => [num(a, 1)?, num(a, 2)?, num(a, 3)?, num(a, 4)?, num(a, 5)?],
        _ => return Err("usage: kbl state <s> | brightness <0-3> | rgb <mode r g b speed>".into()),
    };
    let r = acpi.rgb(rgb[0], rgb[1], rgb[2], rgb[3], rgb[4]);
    println!("rgb={r}");
    if r == 1 { Ok(()) } else { Err("device returned non-success".into()) }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&a) {
        eprintln!("kbl: {e}");
        std::process::exit(1);
    }
}
