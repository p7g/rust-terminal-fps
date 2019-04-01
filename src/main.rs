use std::time::Instant;
use winapi::ctypes::wchar_t;
use winapi::shared::ntdef::NULL;
use winapi::shared::minwindef::DWORD;
use winapi::um::winnt;
use winapi::um::wincon;
use winapi::um::winuser;

const FORWARD: char = 'W';
const BACKWARD: char = 'S';
const LEFT: char = 'A';
const RIGHT: char = 'D';

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 40;

const MAP_HEIGHT: usize = 16;
const MAP_WIDTH: usize = 32;

const DEPTH: f64 = 16f64;
const FOV: f64 = std::f64::consts::PI / 4.0;

fn main() {
    let screen: &mut [wchar_t; SCREEN_WIDTH * SCREEN_HEIGHT] =
        &mut [0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let console = unsafe {
        let console: winnt::HANDLE = wincon::CreateConsoleScreenBuffer(
            winnt::GENERIC_READ | winnt::GENERIC_WRITE,
            0,
            std::ptr::null(),
            wincon::CONSOLE_TEXTMODE_BUFFER,
            NULL,
        );
        wincon::SetConsoleActiveScreenBuffer(console);
        console
    };

    let map: &str = "\
################################\
#..............................#\
#..............................#\
#..............................#\
#..........................#...#\
#..........................#...#\
#..............................#\
#..............................#\
#..............................#\
#..............................#\
#..............................#\
#..............................#\
#.......................########\
#..............................#\
#..............................#\
################################\
";

    let mut bytes_written: DWORD = 0;
    let mut player_x = 8f64;
    let mut player_y = 8f64;
    let mut player_a = 0f64;
    let mut time_1 = Instant::now();
    let mut time_2: Instant;

    loop {
        time_2 = Instant::now();
        let elapsed_time = time_2.duration_since(time_1).as_micros();
        let elapsed_time = elapsed_time as f64 / 1_000_000f64 * 2.0;
        time_1 = time_2;

        unsafe {
            if winuser::GetAsyncKeyState(LEFT as i32) != 0 {
                player_a -= 0.8 * elapsed_time;
            } else if winuser::GetAsyncKeyState(RIGHT as i32) != 0 {
                player_a += 0.8 * elapsed_time;
            }
            if winuser::GetAsyncKeyState(FORWARD as i32) != 0 {
                let delta_x = player_a.sin() * 5.0 * elapsed_time;
                let delta_y = player_a.cos() * 5.0 * elapsed_time;
                player_x += delta_x;
                player_y += delta_y;

                if map.as_bytes()[
                    player_y as usize * MAP_WIDTH + player_x as usize
                ] == b'#' {
                    player_x -= delta_x;
                    player_y -= delta_y;
                }
            } else if winuser::GetAsyncKeyState(BACKWARD as i32) != 0 {
                let delta_x = player_a.sin() * 5.0 * elapsed_time;
                let delta_y = player_a.cos() * 5.0 * elapsed_time;

                player_x -= delta_x;
                player_y -= delta_y;

                if map.as_bytes()[
                    player_y as usize * MAP_WIDTH + player_x as usize
                ] == b'#' {
                    player_x += delta_x;
                    player_y += delta_y;
                }
            }
        }

        for x in 0..SCREEN_WIDTH {
            let ray_angle = (player_a - FOV / 2.0)
                + (x as f64 / SCREEN_WIDTH as f64) * FOV;

            let mut distance_to_wall = 0f64;

            let eye_x = ray_angle.sin();
            let eye_y = ray_angle.cos();

            let mut hit_wall = false;
            let mut boundary = false;
            while !hit_wall && distance_to_wall < DEPTH {
                distance_to_wall += 0.1;

                let test_x = (player_x + eye_x * distance_to_wall) as i64;
                let test_y = (player_y + eye_y * distance_to_wall) as i64;

                if test_x < 0 || test_x >= MAP_WIDTH as i64
                || test_y < 0 || test_y >= MAP_HEIGHT as i64 {
                    hit_wall = true;
                    distance_to_wall = DEPTH;
                } else {
                    if map.as_bytes()[
                        test_y as usize * MAP_WIDTH + test_x as usize
                    ] == b'#' {
                        hit_wall = true;

                        let mut corners: Vec<(f64, f64)> = Vec::new();
                        for tx in 0..2 {
                            for ty in 0..2 {
                                let vy = test_y as f64 + ty as f64 - player_y;
                                let vx = test_x as f64 + tx as f64 - player_x;
                                let d = (vx * vx + vy * vy).sqrt();
                                let dot = eye_x * vx / d + eye_y * vy / d;
                                corners.push((d, dot));
                            }
                        }

                        corners.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                        let bound = 0.005;
                        if corners[0].1.acos() < bound
                        || corners[1].1.acos() < bound {
                            boundary = true;
                        }
                    }
                }
            }

            let ceiling = (
                SCREEN_HEIGHT as f64 / 2.0
                - SCREEN_HEIGHT as f64 / distance_to_wall
            ) as i64;
            let floor = SCREEN_HEIGHT as i64 - ceiling;

            let mut shade = if distance_to_wall <= DEPTH / 4.0 {
                '\u{2588}'
            } else if distance_to_wall < DEPTH / 3.0 {
                '\u{2593}'
            } else if distance_to_wall < DEPTH / 2.0 {
                '\u{2592}'
            } else if distance_to_wall < DEPTH {
                '\u{2591}'
            } else { ' ' };

            if boundary {
                shade = ' ';
            }

            for y in 0..SCREEN_HEIGHT {
                if (y as i64) < ceiling {
                    screen[y * SCREEN_WIDTH + x] = ' ' as wchar_t;
                } else if y as i64 > ceiling && y as i64 <= floor {
                    screen[y * SCREEN_WIDTH + x] = shade as wchar_t;
                } else {
                    let distance = 1.0 - (
                        (y as f64 - SCREEN_HEIGHT as f64 / 2.0)
                        / (SCREEN_HEIGHT as f64 / 2.0)
                    );
                    let shade = if distance < 0.25 {
                        '#'
                    } else if distance < 0.5 {
                        'x'
                    } else if distance < 0.75 {
                        '.'
                    } else if distance < 0.9 {
                        '-'
                    } else { ' ' };
                    screen[y * SCREEN_WIDTH + x] = shade as wchar_t;
                }
            }
        }

        let stats = format!(" X={:3.2}, Y={:3.2}, A={:3.2}  FPS={:3.2} ",
                            player_x, player_y, player_a, 1.0 / elapsed_time);
        for (i, c) in stats.chars().enumerate() {
            screen[i] = c as wchar_t;
        }

        for nx in 0..MAP_WIDTH {
            for ny in 0..MAP_HEIGHT {
                screen[(ny + 1) * SCREEN_WIDTH + nx] =
                    map.as_bytes()[ny * MAP_WIDTH + nx] as wchar_t;
            }
        }

        screen[(player_y as usize + 1) * SCREEN_WIDTH + player_x as usize] =
            'P' as wchar_t;

        screen[SCREEN_WIDTH * SCREEN_HEIGHT - 1] = '\0' as wchar_t;
        unsafe {
            wincon::WriteConsoleOutputCharacterW(
                console,
                &screen[0],
                (SCREEN_WIDTH * SCREEN_HEIGHT) as u32,
                wincon::COORD { X: 0, Y: 0 },
                &mut bytes_written,
            );
        }
    }
}
