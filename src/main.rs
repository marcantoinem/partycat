use hsv::hsv_to_rgb;
use std::{
    fs::File,
    io::{self, stdin, stdout, BufRead, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use termion::{clear, color, cursor, raw::IntoRawMode};

#[derive(Debug)]
struct VisibleStdIn {
    data: Arc<Mutex<String>>,
    width: u16,
    height: u16,
    is_finished: Arc<AtomicBool>,
}

impl Clone for VisibleStdIn {
    fn clone(&self) -> Self {
        let data = Arc::clone(&self.data);
        let width = self.width;
        let height = self.height;
        let is_finished = Arc::clone(&self.is_finished);

        Self {
            data,
            width,
            height,
            is_finished,
        }
    }
}

impl VisibleStdIn {
    fn process(self) {
        for input in stdin().lock().lines() {
            let Ok(input) = input else {continue};

            let mut data = self.data.lock().unwrap();
            data.push_str(&input);
            data.push('\n');
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(1000));
        self.is_finished.store(true, Ordering::Relaxed);
    }
    fn data(&self) -> String {
        let char_capacity = self.len() as usize;
        let len = self.data.lock().unwrap().len();
        self.data.lock().unwrap()[len.saturating_sub(char_capacity)..]
            .chars()
            .collect()
    }
    fn len(&self) -> u32 {
        self.width as u32 * self.height as u32 - 200
    }
    fn new() -> Self {
        let data = Arc::new(Mutex::new(String::new()));
        let (height, width) = termion::terminal_size().unwrap();
        let is_finished = Arc::new(AtomicBool::new(false));

        let visible_std_in = Self {
            data,
            width,
            height,
            is_finished,
        };
        let visible_std_in_ref = visible_std_in.clone();

        thread::spawn(move || visible_std_in_ref.process());
        visible_std_in
    }
    fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::Relaxed)
    }
}

fn main() {
    if !atty::isnt(atty::Stream::Stdin) {
        return;
    }

    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin_buffer = VisibleStdIn::new();
    let mut file = File::create("foo.txt").unwrap();
    let mut hue = 0.0;

    while !stdin_buffer.is_finished() {
        let mut new_string = String::new();
        for c in stdin_buffer.data().chars() {
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            new_string.push_str(&format!("{}{}", color::Fg(color::Rgb(r, g, b)), c));
            hue = (hue + 360.0 / (16 * stdin_buffer.len()) as f64) % 360.0;
        }
        file.write_all(&new_string.as_bytes()).unwrap();
        write!(stdout, "{}{}{}", clear::All, cursor::Goto(1, 1), new_string).unwrap();
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}
