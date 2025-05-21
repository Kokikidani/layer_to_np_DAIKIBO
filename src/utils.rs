use chrono::{ DateTime, Local };
use std::{ fs, io::Error, process, time::SystemTime };
use uuid::Uuid;

mod output;
pub use output::*;

mod algo;
pub use algo::*;

/// ファイル名を指定し、ファイルをString形式で読み込む
/// Result型なので、この関数の外側でエラーハンドリングを行うこと
pub fn read_file(filepath: &str) -> Result<String, Error> {
    let file_contents = fs::read_to_string(filepath)?;
    Ok(file_contents)
}

/// タイムスタンプとUUIDの一部を使用し、IDを生成する
/// 形式は、%Y%m%d/%H%M%S_0%3f_UUID
pub fn generate_id() -> String {
    let current_time = SystemTime::now();
    let timestamp: DateTime<Local> = current_time.into();
    let _time_str = timestamp.format("%Y%m%d/%H%M%S").to_string();

    let pid = process::id();
    let pid_str = format!("{:010}", pid);

    // format!("{}_{}", _time_str, pid_str)
    pid_str.to_string()
}

/// UUIDを生成する
pub fn generate_uuid() -> Uuid {
    Uuid::now_v7()
}

/// CSV形式の二次元配列を読み込む
pub fn string_to_vec2_bool(data: &str) -> Vec<Vec<bool>> {
    let mut o = vec![];

    for l in data.trim().lines() {
        let mut r = vec![];
        for v in l.trim().split(',') {
            if let Ok(b) = v.trim().parse::<usize>() {
                if b == 0 {
                    r.push(false);
                } else {
                    r.push(true);
                }
            }
        }

        o.push(r);
    }

    o
}

pub fn find_x_for_y(x_y: &[(f64, f64)], target_y: f64) -> Option<f64> {
    for i in 0..x_y.len() - 1 {
        let x0 = x_y[i].0;
        let x1 = x_y[i + 1].0;
        let y0 = x_y[i].1;
        let y1 = x_y[i + 1].1;

        if y0 <= target_y && y1 >= target_y {
            let a = (y1 - y0) / (x1 - x0);
            let x_interp = x0 + (target_y - y0) / a;
            return Some(x_interp);
        }
    }

    None
}

pub fn contains_subslice<T: PartialEq>(main_slice: &[T], sub_slice: &[T]) -> bool {
    // main_sliceをsub_sliceの長さのウィンドウでスライドしながら部分一致を探す
    main_slice.windows(sub_slice.len()).any(|window| window == sub_slice)
}