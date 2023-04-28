use std::error::Error;
use rusty_audio::Audio;
use std::{fs, io, thread};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crossterm::{event, ExecutableCommand, terminal};
use crossterm::cursor::Hide;
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use space_invaders::{frame, render};
use space_invaders::frame::{Drawable, new_frame};
use space_invaders::invaders::Invaders;
use space_invaders::player::Player;

fn main() -> Result<(), Box<dyn Error>>{
    let mut audio = Audio::new();
    let audio_files = fs::read_dir("sounds/").unwrap();
    for file in audio_files {
        let path = file.unwrap().path().into_os_string().into_string().unwrap();
        println!("{}", &path.trim_end_matches(".wav")[7..]);
        audio.add(&path.trim_end_matches(".wav")[7..], &path)
    };
    audio.play("startup");

    // Output render
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    //Render Loop in another Thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => {
                    x
                },
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        };
    });

    //Game Loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        // First frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();
        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    },
                    KeyCode::Left => {
                      player.move_left();
                    },
                    KeyCode::Right => {
                        player.move_right();
                    },
                    KeyCode::Char(' ') => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    _ => {}
                }
            }
        }
        //Timer Update
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        // Draw a render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    //Cleanup of thread
    drop(render_tx);
    render_handle.join().unwrap();
    //Cleanup of audio
    audio.wait();
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    return Ok(())
}
