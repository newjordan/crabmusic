// white_rabbit.rs - Follow the White Rabbit through the Digital Looking Glass
// A Matrix-inspired animation using authentic Japanese Katakana, Kanji, and symbols
// "You take the red pill, you stay in Wonderland, and I show you how deep the rabbit hole goes."
// Features the actual Matrix digital rain characters for true hacker aesthetic

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Color as CColor, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_millis(50); // 20 FPS for smooth animation

// Authentic Matrix digital rain characters - Katakana, Kanji, numbers, and symbols
const MATRIX_CHARS: &str = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン日月火水木金土風雨雷電気力光闇空海山川森林花鳥虫魚竜神仏道心愛死生戦平和夢希望運命自由正義悪善美醜強弱大小上下左右前後内外始終古新若老男女子親友敵味方勝負成敗栄枯盛衰喜怒哀楽苦楽生死有無真偽是非可不然否定肯定疑問答弁論理証明仮説実験観察分析総合帰納演繹因果必然偶然確率統計数量単複個全部分集合要素関係構造機能形式内容本質現象存在意識認識感覚知覚思考判断推理記憶想像感情意志行動012345678901234567890+-×÷=()<>[]{}!?@#$%^&*~`|\\/_:;\"',.";

// Box drawing for geometric structures (reserved for future use)
#[allow(dead_code)]
const BOX_CHARS: &[char] = &['╔', '═', '╗', '║', '╚', '╝', '├', '┤', '┬', '┴', '┼', '─', '│'];

// Block characters for depth and shadows (reserved for future use)
#[allow(dead_code)]
const BLOCK_CHARS: &[char] = &['█', '▓', '▒', '░', '▄', '▀', '▌', '▐'];

// Matrix rain column
struct MatrixColumn {
    x: usize,
    y: f32,
    speed: f32,
    chars: Vec<char>,
    length: usize,
    intensity: f32,
}

impl MatrixColumn {
    fn new(x: usize, _height: usize) -> Self {
        let mut rng = rand::thread_rng();
        let matrix_chars: Vec<char> = MATRIX_CHARS.chars().collect();
        let length = rng.gen_range(5..15);
        let mut chars = Vec::new();

        for _ in 0..length {
            chars.push(matrix_chars[rng.gen_range(0..matrix_chars.len())]);
        }

        Self {
            x,
            y: rng.gen_range(-20.0..0.0) as f32,
            speed: rng.gen_range(0.3..1.5),
            chars,
            length,
            intensity: rng.gen_range(0.3..1.0),
        }
    }

    fn update(&mut self, height: usize) {
        self.y += self.speed;
        if self.y > height as f32 + self.length as f32 {
            self.y = -(self.length as i32) as f32;
            // Regenerate characters
            let mut rng = rand::thread_rng();
            let matrix_chars: Vec<char> = MATRIX_CHARS.chars().collect();
            for i in 0..self.chars.len() {
                self.chars[i] = matrix_chars[rng.gen_range(0..matrix_chars.len())];
            }
            self.intensity = rng.gen_range(0.3..1.0);
        }
    }

    fn draw(&self, buffer: &mut [Vec<(char, CColor)>], height: usize) {
        for (i, &ch) in self.chars.iter().enumerate() {
            let y_pos = self.y as i32 + i as i32;
            if y_pos >= 0 && y_pos < height as i32 {
                let fade = 1.0 - (i as f32 / self.length as f32);
                let green = (255.0 * fade * self.intensity) as u8;
                let color = CColor::Rgb { r: 0, g: green, b: green / 4 };
                buffer[y_pos as usize][self.x] = (ch, color);
            }
        }
    }
}

// The White Rabbit - multiple representations
struct WhiteRabbit {
    phase: f32,
    x: f32,
    y: f32,
    state: usize,
    pulse: f32,
}

impl WhiteRabbit {
    fn new(width: usize, height: usize) -> Self {
        Self {
            phase: 0.0,
            x: width as f32 / 2.0,
            y: height as f32 / 2.0,
            state: 0,
            pulse: 0.0,
        }
    }

    fn update(&mut self, width: usize, height: usize, time: f32) {
        self.phase += 0.05;
        self.pulse = (time * 2.0).sin().abs();

        // Move in a figure-8 pattern
        self.x = width as f32 / 2.0 + (self.phase.sin() * width as f32 * 0.3);
        self.y = height as f32 / 2.0 + ((self.phase * 2.0).sin() * height as f32 * 0.2);

        // Change state every 2 seconds
        self.state = ((time / 2.0) as usize) % 5;
    }

    fn get_rabbit_art(&self) -> Vec<&str> {
        match self.state {
            0 => vec![  // Kanji rabbit - "White Rabbit" (白兎)
                "  白兎  ",
                "〈○ ○〉",
                " ╰━━╯ ",
                "  ∪∪  ",
            ],
            1 => vec![  // Block character rabbit with Japanese
                " ▄兎▄ ",
                "█◉ ◉█",
                "▐▌▀▀▐▌",
                " ▀月▀ ",
            ],
            2 => vec![  // Matrix code rabbit
                "╭─◯◯─╮",
                "│夢╰╯夢│",
                "├──┬─┤",
                "╰──┴─╯",
            ],
            3 => vec![  // Mixed Matrix rabbit
                " 「白兎」",
                "⟨█▓▓█⟩",
                " ▒══▒ ",
                " ░運░ ",
            ],
            _ => vec![  // Digital glitch rabbit
                "  /\\  /\\  ",
                " (◉ ◉) ",
                "  (><)  ",
                " 〔〕〔〕",
            ],
        }
    }

    fn draw(&self, buffer: &mut [Vec<(char, CColor)>], width: usize, height: usize) {
        let art = self.get_rabbit_art();
        let start_x = (self.x as i32) - (art[0].len() as i32 / 2);
        let start_y = (self.y as i32) - (art.len() as i32 / 2);

        // Draw shadow first
        for (dy, line) in art.iter().enumerate() {
            for (dx, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let sx = start_x + dx as i32 + 2;
                    let sy = start_y + dy as i32 + 1;
                    if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                        buffer[sy as usize][sx as usize] = ('░', CColor::Rgb { r: 20, g: 20, b: 30 });
                    }
                }
            }
        }

        // Draw rabbit with pulsing effect
        let pulse_color = (200.0 + self.pulse * 55.0) as u8;
        for (dy, line) in art.iter().enumerate() {
            for (dx, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let rx = start_x + dx as i32;
                    let ry = start_y + dy as i32;
                    if rx >= 0 && rx < width as i32 && ry >= 0 && ry < height as i32 {
                        let color = match self.state {
                            0 => CColor::Rgb { r: pulse_color, g: pulse_color, b: pulse_color },
                            1 => CColor::Rgb { r: pulse_color, g: pulse_color / 2, b: pulse_color },
                            2 => CColor::Rgb { r: pulse_color / 2, g: pulse_color, b: pulse_color },
                            3 => CColor::Rgb { r: pulse_color, g: pulse_color, b: pulse_color / 2 },
                            _ => CColor::Rgb { r: pulse_color, g: pulse_color, b: pulse_color },
                        };
                        buffer[ry as usize][rx as usize] = (ch, color);
                    }
                }
            }
        }
    }
}

// Geometric portal effects using box drawing characters
struct Portal {
    x: usize,
    y: usize,
    radius: f32,
    phase: f32,
    active: bool,
}

impl Portal {
    fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            radius: 0.0,
            phase: 0.0,
            active: false,
        }
    }

    fn activate(&mut self) {
        self.active = true;
        self.radius = 0.0;
    }

    fn update(&mut self) {
        if self.active {
            self.phase += 0.1;
            self.radius += 0.5;
            if self.radius > 20.0 {
                self.radius = 0.0;
                self.active = false;
            }
        }
    }

    fn draw(&self, buffer: &mut [Vec<(char, CColor)>], width: usize, height: usize) {
        if !self.active {
            return;
        }

        let intensity = 1.0 - (self.radius / 20.0);
        let color = CColor::Rgb {
            r: (150.0 * intensity) as u8,
            g: (100.0 * intensity) as u8,
            b: (255.0 * intensity) as u8,
        };

        // Draw expanding box
        let r = self.radius as i32;
        for dx in -r..=r {
            for dy in -r..=r {
                let px = self.x as i32 + dx;
                let py = self.y as i32 + dy;

                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    // Only draw the outline
                    if dx.abs() == r || dy.abs() == r {
                        let ch = if dx.abs() == r && dy.abs() == r {
                            if dx < 0 && dy < 0 { '╔' }
                            else if dx > 0 && dy < 0 { '╗' }
                            else if dx < 0 && dy > 0 { '╚' }
                            else { '╝' }
                        } else if dx.abs() == r {
                            '║'
                        } else {
                            '═'
                        };
                        buffer[py as usize][px as usize] = (ch, color);
                    }
                }
            }
        }
    }
}

// Message display with typewriter effect
struct Message {
    text: &'static str,
    revealed: usize,
    timer: f32,
    complete: bool,
}

impl Message {
    fn new(text: &'static str) -> Self {
        Self {
            text,
            revealed: 0,
            timer: 0.0,
            complete: false,
        }
    }

    fn update(&mut self) {
        if !self.complete {
            self.timer += 1.0;
            if self.timer > 2.0 {
                self.revealed = (self.revealed + 1).min(self.text.len());
                self.timer = 0.0;
                if self.revealed >= self.text.len() {
                    self.complete = true;
                }
            }
        }
    }

    fn draw(&self, buffer: &mut [Vec<(char, CColor)>], y: usize, width: usize) {
        let start_x = (width - self.text.len()) / 2;
        for (i, ch) in self.text.chars().take(self.revealed).enumerate() {
            if start_x + i < width {
                buffer[y][start_x + i] = (ch, CColor::Rgb { r: 200, g: 200, b: 255 });
            }
        }
    }
}

// Main animation controller
struct Animation {
    width: usize,
    height: usize,
    time: f32,
    matrix_rain: Vec<MatrixColumn>,
    rabbit: WhiteRabbit,
    portal: Portal,
    messages: Vec<Message>,
    current_message: usize,
}

impl Animation {
    fn new(width: usize, height: usize) -> Self {
        let mut matrix_rain = Vec::new();
        for x in (0..width).step_by(2) {
            matrix_rain.push(MatrixColumn::new(x, height));
        }

        let messages = vec![
            Message::new("FOLLOW THE WHITE RABBIT"),
            Message::new("DOWN THE RABBIT HOLE"),
            Message::new("WAKE UP, NEO..."),
            Message::new("THE MATRIX HAS YOU"),
            Message::new("KNOCK, KNOCK..."),
        ];

        Self {
            width,
            height,
            time: 0.0,
            matrix_rain,
            rabbit: WhiteRabbit::new(width, height),
            portal: Portal::new(width / 2, height / 2),
            messages,
            current_message: 0,
        }
    }

    fn update(&mut self) {
        self.time += 0.05;

        // Update matrix rain
        for col in &mut self.matrix_rain {
            col.update(self.height);
        }

        // Update rabbit
        self.rabbit.update(self.width, self.height, self.time);

        // Update portal
        self.portal.update();

        // Activate portal periodically
        if (self.time as i32) % 5 == 0 && (self.time * 20.0) as i32 % 20 == 0 {
            self.portal.x = self.rabbit.x as usize;
            self.portal.y = self.rabbit.y as usize;
            self.portal.activate();
        }

        // Update messages
        if self.current_message < self.messages.len() {
            self.messages[self.current_message].update();
            if self.messages[self.current_message].complete {
                if self.current_message < self.messages.len() - 1 {
                    self.current_message += 1;
                } else {
                    // Loop back to the beginning
                    self.current_message = 0;
                    for msg in &mut self.messages {
                        msg.revealed = 0;
                        msg.complete = false;
                        msg.timer = 0.0;
                    }
                }
            }
        }
    }

    fn render(&self) -> Vec<Vec<(char, CColor)>> {
        let mut buffer = vec![vec![(' ', CColor::Black); self.width]; self.height];

        // Draw matrix rain background
        for col in &self.matrix_rain {
            col.draw(&mut buffer, self.height);
        }

        // Draw portal
        self.portal.draw(&mut buffer, self.width, self.height);

        // Draw rabbit
        self.rabbit.draw(&mut buffer, self.width, self.height);

        // Draw current message
        if self.current_message < self.messages.len() {
            self.messages[self.current_message].draw(&mut buffer, 2, self.width);
        }

        // Add decorative borders using box drawing characters
        for x in 0..self.width {
            buffer[0][x] = ('═', CColor::Rgb { r: 50, g: 50, b: 100 });
            buffer[self.height - 1][x] = ('═', CColor::Rgb { r: 50, g: 50, b: 100 });
        }
        for row in buffer.iter_mut().take(self.height) {
            row[0] = ('║', CColor::Rgb { r: 50, g: 50, b: 100 });
            row[self.width - 1] = ('║', CColor::Rgb { r: 50, g: 50, b: 100 });
        }
        buffer[0][0] = ('╔', CColor::Rgb { r: 50, g: 50, b: 100 });
        buffer[0][self.width - 1] = ('╗', CColor::Rgb { r: 50, g: 50, b: 100 });
        buffer[self.height - 1][0] = ('╚', CColor::Rgb { r: 50, g: 50, b: 100 });
        buffer[self.height - 1][self.width - 1] = ('╝', CColor::Rgb { r: 50, g: 50, b: 100 });

        // Add glitch effect occasionally
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < 0.05 {
            for _ in 0..10 {
                let gx = rng.gen_range(0..self.width);
                let gy = rng.gen_range(0..self.height);
                let glitch_chars = ['▓', '▒', '░', '█', '▄', '▀'];
                let ch = glitch_chars[rng.gen_range(0..glitch_chars.len())];
                buffer[gy][gx] = (ch, CColor::Rgb {
                    r: rng.gen_range(100..255),
                    g: rng.gen_range(0..100),
                    b: rng.gen_range(100..255),
                });
            }
        }

        buffer
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Clear(ClearType::All),
        Hide
    )?;

    let (width, height) = terminal::size()?;
    let mut animation = Animation::new(width as usize, height as usize);

    let start_time = Instant::now();

    loop {
        // Check for exit key
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        // Update animation
        animation.update();

        // Render frame
        let buffer = animation.render();

        // Draw to terminal
        for (y, row) in buffer.iter().enumerate() {
            execute!(stdout, MoveTo(0, y as u16))?;
            for (ch, color) in row {
                execute!(
                    stdout,
                    SetForegroundColor(*color),
                    Print(ch)
                )?;
            }
        }

        stdout.flush()?;

        // Control frame rate
        std::thread::sleep(FRAME_DURATION);

        // Loop after ~12 seconds (240 frames at 20fps)
        if start_time.elapsed().as_secs() >= 12 {
            animation = Animation::new(width as usize, height as usize);
        }
    }

    // Cleanup
    execute!(
        stdout,
        ResetColor,
        Show,
        LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;

    println!("\n\"There is no spoon.\" 🐇\n");

    Ok(())
}