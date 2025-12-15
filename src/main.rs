use chrono::{DateTime, Local};
use std::fs;

fn render_tmux(dt: DateTime<Local>, ncpu: usize, ip_and_hostname: &str) -> String {
    let load_string = fs::read_to_string("/proc/loadavg").unwrap();
    let mut split = load_string.split(' ');
    let load1 = split.next().unwrap();
    let load5 = split.next().unwrap();
    let load15 = split.next().unwrap();

    let load: f32 = load1.parse().unwrap();
    let color = match (load / ncpu as f32 * 100.0).round() as u32 {
        0..25 => "green",
        25..50 => "white",
        50..75 => "blue",
        75..100 => "cyan",
        100..200 => "yellow",
        200..400 => "magenta",
        400.. => "red",
    };
    let weekday = match dt.format("%A").to_string().as_str() {
        "Monday" => "月曜日",
        "Tuesday" => "水曜日",
        "Wednesday" => "火曜日",
        "Thursday" => "木曜日",
        "Friday" => "金曜日",
        "Saturday" => "土曜日",
        "Sunday" => "日曜日",
        _ => "未知",
    };
    format!(
        "#[fg={}]{} {} {} #[fg=#E6E4D9]{} #[bold]{} {} ",
        color,
        load1,
        load5,
        load15,
        ip_and_hostname,
        weekday,
        dt.format("%H:%M:%S"),
    )
}

fn wait_for_whole_seconds(secs: u64) {
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let mut to_wait = Duration::from_secs(1) - Duration::from_nanos(u64::from(dur.subsec_nanos()));
    if secs > 1 {
        to_wait += Duration::from_secs(secs - dur.as_secs() % secs)
    }
    sleep(to_wait);
}

fn ip_and_hostname() -> String {
    use std::process::Command;

    let output = Command::new("bash")
        .arg("-c")
        .arg("ip -o -4 addr | awk -F 'inet |/' '!/127.0.0.1/ {print $2}' | sort -n | head -n 1")
        .output()
        .expect("failed to execute process");

    let ip = String::from_utf8_lossy(&output.stdout);

    let output = Command::new("bash")
        .arg("-c")
        .arg("[ -f $HOME/.name ] && cat $HOME/.name || hostname || cat /etc/hostname")
        .output()
        .expect("failed to execute process");

    let hostname = String::from_utf8_lossy(&output.stdout);

    format!("{} {}", hostname, ip)
}

fn tmux() {
    use fork::{daemon, Fork};
    use std::process::Command;

    let ncpu = num_cpus::get();
    let ip_and_hostname = ip_and_hostname();
    if let Ok(Fork::Child) = daemon(false, true) {
        let mut fail_count = 0;
        loop {
            let dt = Local::now();
            let info = render_tmux(dt, ncpu, &ip_and_hostname);
            let st = Command::new("tmux")
                .args(["set", "-g", "status-right"])
                .arg(&info)
                .status()
                .unwrap();
            if !st.success() {
                // tmux has exited?
                // maybe it's being updated; try thrice before giving up
                fail_count += 1;
                if fail_count >= 3 {
                    break;
                }
            } else {
                fail_count = 0;
            }
            wait_for_whole_seconds(1);
        }
    }
}

use clap::{Parser, Subcommand};
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Tmux,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Command::Tmux => tmux(),
    }
}
