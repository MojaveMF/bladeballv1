use std::{ time::{ self, Duration }, thread, str::FromStr };

use config::{ ConfigType, RgbColor };
use enigo::MouseControllable;
use screenshots::{ Screen, image::{ ColorType, Rgb } };
use sysinfo::{ SystemExt, ProcessExt, Pid };
use tracing::info;
use windows_sys::Win32::System::Diagnostics::Debug::Beep;

mod config;

static mut SHOULD_CHECK: bool = false;

fn toggle_should_check() {
    unsafe {
        SHOULD_CHECK = !SHOULD_CHECK;
        info!("Toggled {}", SHOULD_CHECK)
    }
}

fn beep(hertz: u32, millis: u32) {
    unsafe {
        Beep(hertz, millis);
    }
}

fn vector2_magnitude(x: i32, y: i32) -> f32 {
    let sum = x * x + y * y;
    return f32::sqrt(sum as f32);
}

fn vector3_magnitude(x: i32, y: i32, z: i32) -> f32 {
    let sum = x * x + y * y + z * z;
    return f32::sqrt(sum as f32);
}

fn get_mean(arr: &[u32]) -> u32 {
    let mut total = 0;

    for num in arr {
        total += num;
    }

    total / (arr.len() as u32)
}

async fn async_start_scanner(display: Screen, cfg: ConfigType) {
    return start_scanner(display, cfg);
}

/* 
fn get_color_distance(color1: RgbColor, color2: RgbColor) -> f32 {
    let r3 = ((color2.r as i32) - (color1.r as i32)) ^ 2;
    let g3 = ((color2.g as i32) - (color1.g as i32)) ^ 2;
    let b3 = ((color2.b as i32) - (color1.b as i32)) ^ 2;
    f32::sqrt((r3 + g3 + b3) as f32)
}
*/

fn get_color_distance(min: RgbColor, max: RgbColor) -> f32 {
    let distance = vector3_magnitude(
        (min.r as i32) - (max.r as i32),
        (min.g as i32) - (max.g as i32),
        (min.b as i32) - (max.b as i32)
    );
    return distance;
}

fn start_scanner(display: Screen, cfg: ConfigType) {
    let capture = display.capture().expect("Failed to capture display");

    let height = display.display_info.height;
    let width = display.display_info.width;

    let mut x_vec = vec![];
    let mut y_vec = vec![];

    for x in 0..width {
        for y in 0..height {
            let pixel = capture.get_pixel(x, y);

            let pxl = config::RgbColor {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
            };

            let distance = get_color_distance(pxl, cfg.target_color);
            if -cfg.color_range < distance && distance < cfg.color_range {
                x_vec.push(x);
                y_vec.push(y);
            }
        }
    }

    let x_len = x_vec.len();
    if x_len < (cfg.minimum_density as usize) && cfg.checks.density {
        return;
    }

    /* Get positions and convert them to signed int */
    let x = get_mean(&x_vec) as i32;
    let y = get_mean(&y_vec) as i32;

    let center_x = (height / 2) as i32;
    let center_y = (height / 2) as i32;

    let magnitude = vector2_magnitude(center_x - x, center_y - y);

    if magnitude >= (cfg.check_radius as f32) && cfg.checks.radius {
        return;
    }

    let mut enigo = enigo::Enigo::new();
    enigo.mouse_click(enigo::MouseButton::Left);
    if cfg.log_hits {
        info!("Target Found, Desnity={} Position={},{} ", x_len, x, y);
    }
}

fn wait_for_process<T: Into<String>>(process_name: T) -> Pid {
    let program_name: String = process_name.into();

    loop {
        let current_system = sysinfo::System::new_all();
        for process in current_system.processes_by_exact_name(&program_name) {
            let name = process.name();
            let pid = process.pid();
            info!("Found target program {} [PID={}]", name, pid);
            return pid;
        }
        std::thread::sleep(time::Duration::from_millis(10));
    }
}

fn sync_main(cfg: ConfigType) {
    let screen = screenshots::Screen::all().expect("Couldnt find display");
    let display = screen[0];

    loop {
        unsafe {
            if SHOULD_CHECK {
                thread::sleep(Duration::from_secs_f32((1 as f32) / (cfg.check_cycles as f32)));

                let cfg = cfg.clone();
                let display = display.clone();

                thread::spawn(move || {
                    start_scanner(display, cfg);
                });
            }
        }
    }
}

#[tokio::main]
async fn async_main(cfg: ConfigType) {
    let screen = screenshots::Screen::all().expect("Couldnt find display");
    let display = screen[0];

    loop {
        unsafe {
            if SHOULD_CHECK {
                thread::sleep(Duration::from_secs_f32((1 as f32) / (cfg.check_cycles as f32)));
                let cfg = cfg.clone();
                let display = display.clone();

                tokio::task::spawn(async_start_scanner(display, cfg));
            }
        }
    }
}

fn enabled_toggle(toggle_key: String) {
    let key = mki::Keyboard::from_str(&toggle_key).expect("Bad key in Config");
    let key_dupe = key.clone();

    key.bind(move |_| {
        if key_dupe.is_pressed() {
            toggle_should_check();
            beep(1000, 250)
        }
    });
}

fn main() {
    info!("Made by mojavemf on discord (https://discord.gg/wARqb5aCuS)");
    tracing_subscriber::fmt().init();
    let cfg = config::ConfigType::load().expect("Couldnt load config");
    info!("Loaded config: {:?}", cfg);
    enabled_toggle(cfg.toggle_key.clone());
    wait_for_process("RobloxPlayerBeta.exe");

    match cfg.use_tokio {
        true => {
            return async_main(cfg);
        }
        false => {
            return sync_main(cfg);
        }
    }
}
