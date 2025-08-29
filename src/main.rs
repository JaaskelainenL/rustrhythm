use rfd::FileDialog;
use sdl2::render::{TextureQuery, BlendMode};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};
use sdl2::mixer::{InitFlag, Music};
use sdl2::image::LoadTexture;
use std::collections::HashSet;
use std::{fs, env};


mod game_state;
use game_state::GameState;


fn main() {

    // CONFIGS
    let map_path = FileDialog::new().set_directory(env::current_dir().expect("Failed to get current directory"))
        .set_title("Select a folder with an .sm file in it")
        .pick_folder()
        .expect("Choose a folder");

    let game_speed = 1.5; // Higher = slower
    let note_gap = 25; // px between the notes


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();



    

    sdl2::mixer::open_audio(44100, sdl2::mixer::DEFAULT_FORMAT, 2, 1024).unwrap();
    let _mixer_context = sdl2::mixer::init(InitFlag::MP3).unwrap();
    sdl2::mixer::allocate_channels(4);


    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let font = ttf_context.load_font("./assets/OpenSans.ttf", 32).unwrap();


    let paths = fs::read_dir(map_path.clone()).unwrap();
    let mut sm_file = String::new(); 
    for path in paths {
        let path = path.unwrap();
        let file_name = path.file_name().into_string().unwrap_or(String::new()); 

        if file_name.ends_with(".sm") {
            sm_file = path.path().to_str().unwrap_or_default().to_string();
        }
    }

    let mut game_state = GameState::new(&sm_file, game_speed);

    let song_name = map_path.clone().join(&game_state.song);


    let music = Music::from_file(song_name).expect("Failed to load music file");

    let window_title = &(game_state.artist.clone() + " - " + &game_state.title);

    let window = video_subsystem.window(window_title, 800, 600)
        .position_centered()
        .vulkan()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let texture_creator = canvas.texture_creator();


    let bg_name = map_path.join(&game_state.bg); 
    let background_img = texture_creator.load_texture(&bg_name).unwrap();



    let mut event_pump = sdl_context.event_pump().unwrap();

    let difficulty = choose_difficulty(&mut canvas, &mut event_pump, &font, &game_state, &background_img, &music);

    game_state.start(difficulty);



    let judgment_line_y: f64 = 550.0;
    let spawn_y: f64 = -50.0;
    let mut score = 0;
    let mut combo = 0;

    let key_to_lane = [
        (Keycode::D, 0),
        (Keycode::F, 1),
        (Keycode::J, 2),
        (Keycode::K, 3),
    ];

    let mut held_keys: HashSet<usize> = HashSet::new();

    let mut music_started = false;


    'running: loop {
        if !music_started && Instant::now() >= game_state.start_time {
            music.play(0).expect("Failed to play music");
            music_started = true;
        }


        // Handle inputs
        let mut pressed_keys = HashSet::new();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(k), .. } => {
                    if let Some(&(_, lane)) = key_to_lane.iter().find(|&&(key, _)| key == k) {
                        if !held_keys.contains(&lane){
                            pressed_keys.insert(lane);
                            held_keys.insert(lane);
                        }
                    }
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    if let Some(&(_, lane)) = key_to_lane.iter().find(|&&(key, _)| key == k) {
                        pressed_keys.remove(&lane);
                        held_keys.remove(&lane);
                    }
                }
                _ => {}
            }
        }

        canvas.clear();

        // Draw BG
        canvas.copy(&background_img, None, None).unwrap();
        canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0,196));
        canvas.fill_rect(Rect::new(175, 0, 400, 600)).unwrap();


        // Draw score
        let surface = font
        .render(&score.to_string())
        .blended(sdl2::pixels::Color::RGB(255, 255, 255))
        .map_err(|e| e.to_string()).unwrap();
        let score_tex = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        let TextureQuery { width, height, .. } = score_tex.query();


        canvas.copy(&score_tex, None, Rect::new(15,15,width,height)).unwrap();


        // Draw combo
        let surface = font
        .render(&combo.to_string())
        .blended(sdl2::pixels::Color::RGBA(255, 255, 255,128))
        .map_err(|e| e.to_string()).unwrap();
        let combo_tex = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        let TextureQuery { width, height, .. } = combo_tex.query();

        canvas.copy(&combo_tex, None, Rect::new(400-(width as i32)/2 -(note_gap/2),(judgment_line_y-100.0) as i32,width,height)).unwrap();        




        // Draw controls
        for (i, &x) in [300-2*note_gap, 350-note_gap, 400, 450+note_gap].iter().enumerate() {
            if held_keys.contains(&i) {
                canvas.set_draw_color(sdl2::pixels::Color::RGB(200, 200, 255));
            } else {
                canvas.set_draw_color(sdl2::pixels::Color::RGB(100, 100, 100));
            }
            canvas.fill_rect(Rect::new(x, judgment_line_y as i32, 50, 25)).unwrap();
        }

        let elapsed_time = game_state.start_time.elapsed().as_secs_f64();


        // Draw arrows
        let mut taken_lanes = [false,false,false,false];
        game_state.arrows.retain(|arrow| {
            if elapsed_time < arrow.spawn_time {
                return true
            }

            let progress = (elapsed_time - arrow.spawn_time) / (arrow.hit_time - arrow.spawn_time);
            let y_pos = spawn_y + progress * (judgment_line_y - spawn_y);
            let x_pos = 300 + (arrow.lane as i32)*50 + (arrow.lane as i32 -2) * note_gap;

            let mut long_pressed = false;

            if arrow.long && held_keys.contains(&arrow.lane){
                if elapsed_time >= arrow.hit_time && elapsed_time <= arrow.end_time {
                    score += 5;
                    long_pressed = true;
                }                
            } else if pressed_keys.contains(&arrow.lane) {
                    let distance = (y_pos - judgment_line_y).abs();
                    if distance < 50.0 && !taken_lanes[arrow.lane] {
                        combo += 1;
                        taken_lanes[arrow.lane] = true;
                        score += match distance {
                            d if d < 5.0 => 500, // Flawless
                            d if d < 10.0 => 300,  // Perfect
                            d if d < 20.0 => 200, // Good
                            _ => 100,             // OK
                        };
                        return false;
                    }
            }

            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));

            if arrow.long {
                let note_y = y_pos as i32;
                let note_height = ((arrow.end_time - arrow.hit_time) / (arrow.hit_time - arrow.spawn_time)) * (judgment_line_y - spawn_y);
            
                let mut rect_height = note_height.abs() as u32; 
                let rect_y = note_y - rect_height as i32;

                if long_pressed {
                    rect_height = (judgment_line_y - (rect_y as f64)) as u32;
                    canvas.set_draw_color(sdl2::pixels::Color::RGB(200, 200, 255));
                }
            
                canvas.fill_rect(Rect::new(x_pos + 10, rect_y, 30, rect_height)).unwrap();
                canvas.fill_rect(Rect::new(x_pos+5, rect_y+(rect_height as i32), 40, 10)).unwrap();

                if long_pressed{
                    rect_y < judgment_line_y  as i32 - 5
                } else {
                    rect_y < 600
                }
            } else{
                canvas.fill_rect(Rect::new(x_pos, y_pos as i32, 50, 25)).unwrap();


                if y_pos >= 600.0{
                    combo = 0;
                    return false;
                }
                true
            }
        });

        if !Music::is_playing() && game_state.arrows.is_empty(){
            break 'running;
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

}


fn choose_difficulty(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, 
                     event_pump: &mut sdl2::EventPump,
                     font: &sdl2::ttf::Font, 
                     game_state: &GameState,
                     background_img: &sdl2::render::Texture<'_>,
                     music: &Music<'static>
                    ) -> u32 {

    let texture_creator = canvas.texture_creator();
    let mut selected: u32 = 0;
    let difficulties = game_state.difficulties.clone();

    // Text surface and texture
    let surface = font
    .render("Choose a difficulty")
    .blended(sdl2::pixels::Color::RGBA(255, 255, 255,0))
    .map_err(|e| e.to_string()).unwrap();
    let intro_tex = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string()).unwrap();

    let TextureQuery { width, height, .. } = intro_tex.query();



    music.play(-1).expect("Failed to play music");
    sdl2::mixer::Music::set_pos(game_state.sample_start).unwrap();

    let mut last_seek = Instant::now();

    'menu: loop {
        if last_seek.elapsed() >= Duration::from_secs_f64(game_state.sample_len){
            sdl2::mixer::Music::set_pos(game_state.sample_start).unwrap();
            last_seek = Instant::now();
        }

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => std::process::exit(0),
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Up), .. } => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Down), .. } => {
                    if selected < (difficulties.len() - 1).try_into().unwrap() {
                        selected += 1;
                    }
                }
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Return), .. } => {
                    break 'menu;
                }
                _ => {}
            }
        }
        // Draw start
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&background_img, None, None).unwrap();

        // Draw text
        canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 128));
        let rect = Rect::new(400-(width as i32 + 50)/2, 50, width+50, height+10);
        canvas.fill_rect(rect).unwrap();

        canvas.copy(&intro_tex, None, Rect::new(400 - (width as i32)/2, 50,width,height)).unwrap();        



        for (i, diff) in difficulties.iter().enumerate() {
            let rect_y = 150 + (i as i32 * 80);

            // highlight selected
            if i == selected as usize {
                canvas.set_draw_color(sdl2::pixels::Color::RGB(200, 200, 255));
            } else {
                canvas.set_draw_color(sdl2::pixels::Color::RGB(100, 100, 100));
            }
            let rect = Rect::new(250, rect_y, 300, 60);
            canvas.fill_rect(rect).unwrap();

            let surface = font
                .render(diff)
                .blended(sdl2::pixels::Color::RGB(255, 255, 255))
                .unwrap();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            let TextureQuery { width, height, .. } = texture.query();

            let text_x = rect.x + (rect.width() as i32 - width as i32) / 2;
            let text_y = rect.y + (rect.height() as i32 - height as i32) / 2;

            canvas.copy(&texture, None, Some(Rect::new(text_x, text_y, width, height))).unwrap();


        }

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    sdl2::mixer::Music::halt();

    selected
}