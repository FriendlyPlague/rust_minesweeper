use std::{thread,time::Duration,cmp, collections::{HashSet, VecDeque}};
use macroquad::{window::{self, screen_width, screen_height},shapes,color, input, time, text, rand, texture};

const GAME_WIDTH: usize = 25;
const GAME_HEIGHT: usize = 25;
const NUM_BOMBS:u32 = 105;

// (isbomb, flagged)
type Tile = (bool,bool);

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: "Minesweeper".to_owned(),
        window_width: 880,
        window_height: 950,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Calculate offsets and scale
    let x_offset = window::screen_width()/40.0;
    let y_offset = window::screen_height()/10.0;
    let scale = {
        let x_scale: f32 = (window::screen_width()-x_offset*2.0)/(GAME_WIDTH as f32);
        let y_scale: f32 = (window::screen_height()-(y_offset+x_offset))/(GAME_HEIGHT as f32);
        let scale = {
            if x_scale < y_scale {x_scale}
            else {y_scale}};
        if scale < 10.0 {10.0}
        else {scale}
    };
    

    // loads texture(s)use indexmap::IndexMap;
    let flag_img: texture::Texture2D = texture::load_texture("assets/flag.png").await.unwrap();
    let landmine_img: texture::Texture2D = texture::load_texture("assets/landmine.png").await.unwrap();

    // creates board and places bomb
    
    let mut game_started = false;
    let mut game_over = false;
    let mut won = false;
    let mut timer_offset = 0.0;
    let mut display_time = String::from("00:00");
    let mut game_board: Vec<Option<Tile>> = Vec::new();
    game_board.resize(GAME_WIDTH*GAME_HEIGHT, Some((false,false)));
    loop {
        // Process input
        let mouse_pos = input::mouse_position();
        let mpg = {
            if mouse_pos.0 > x_offset && mouse_pos.1 > y_offset {
                let ax = ((mouse_pos.0-x_offset)/scale) as usize;
                let ay = ((mouse_pos.1-y_offset)/scale) as usize;
                (ax,ay)
            }
            else{(999999,999999)}
        };
        if input::is_mouse_button_pressed(input::MouseButton::Left) {
            if mpg.0 < GAME_WIDTH && mpg.1 < GAME_HEIGHT && !game_over {
                if !game_started {
                    let x = (mpg.0) as i32 -1;
                    let y = (mpg.1) as i32 -1;
                    for ly in cmp::max(y, 0)..cmp::min(y+3, GAME_HEIGHT as i32) {
                        for lx in cmp::max(x, 0)..cmp::min(x+3, GAME_WIDTH as i32) {
                            game_board[(ly as usize)*GAME_WIDTH+(lx as usize)] = None;
                        }
                    }
                    gen_bombs(NUM_BOMBS, &mut game_board);
                    bfs_destruction(mpg.0 as i32, mpg.1 as i32, &mut game_board);
                    timer_offset = time::get_time();
                    game_started = true;
                }
                match game_board[mpg.1*GAME_WIDTH+mpg.0] {
                    Some((false,false)) => bfs_destruction(mpg.0 as i32, mpg.1 as i32, &mut game_board),
                    Some((false,true)) => {game_board[mpg.1*GAME_WIDTH+mpg.0].replace((false,false));},
                    Some((true,true)) => {game_board[mpg.1*GAME_WIDTH+mpg.0].replace((true,false));},
                    Some((true,false)) => {game_over = true;},
                    None => (),
                }
            }
            if have_won(&game_board) {
                game_over = true;
                won = true;
            }
        }
        else if input::is_mouse_button_pressed(input::MouseButton::Right) && mpg.0 < GAME_WIDTH && mpg.1 < GAME_HEIGHT && mpg.0 != 999999 && !game_over { 
            match game_board[mpg.1*GAME_WIDTH+mpg.0] {
                Some((false,false)) => {game_board[mpg.1*GAME_WIDTH+mpg.0].replace((false,true));},
                Some((false,true)) => {game_board[mpg.1*GAME_WIDTH+mpg.0].replace((false,false));},
                Some((true,true)) => {game_board[mpg.1*GAME_WIDTH+mpg.0].replace((true,false));},
                Some((true,false)) => {game_board[mpg.1*GAME_WIDTH+mpg.0].replace((true,true));},
                None => (),
            }
        }

        if input::is_key_pressed(input::KeyCode::R) {
            game_started = false;
            game_over = false;
            won = false;
            display_time = String::from("00:00");
            reset_board(&mut game_board);
        }
        // RENDER BLOCK
        window::clear_background(color::BROWN);
        for y in 0..GAME_WIDTH {
            for x in 0..GAME_HEIGHT {
                let r_color = {
                    if game_board[y*GAME_WIDTH+x].is_none() {color::BEIGE}
                    else {color::LIGHTGRAY}
                };
                shapes::draw_rectangle((x as f32)*scale+x_offset, (y as f32)*scale+y_offset, scale-1.0, scale-1.0, r_color);
                if game_over {
                    if let Some((true,_)) = game_board[y*GAME_WIDTH+x] {
                        texture::draw_texture(landmine_img, (x as f32)*scale+x_offset, (y as f32)*scale+y_offset, color::BEIGE);
                    }
                }
                else {
                   if let Some((_,true)) = game_board[y*GAME_WIDTH+x] {
                        texture::draw_texture(flag_img, (x as f32)*scale+x_offset, (y as f32)*scale+y_offset, color::RED);
                    }
                    if game_board[y*GAME_WIDTH+x].is_none() {
                        let n_bombs = surrounding_bombs(x as i32, y as i32, &game_board);
                        let n_color ={
                            match n_bombs {
                                0 => color::BLANK,
                                1 => color::BLUE,
                                2 => color::GREEN,
                                3 => color::RED,
                                4 => color::MAGENTA,
                                5 => color::GOLD,
                                6 => color::PURPLE,
                                7=> color::BLACK,
                                8 => color::LIGHTGRAY,
                                _ => color::WHITE,
                            }
                        };
                        text::draw_text(&n_bombs.to_string(), (x as f32)*scale+x_offset+scale*0.25, (y as f32)*scale+y_offset+scale*0.8, scale, n_color);
                    }
                }
            }

            // Draw timer
            if !game_over && game_started {
                let minutes = (time::get_time()-timer_offset).floor() as usize / 60;
                let minutes = {
                    if minutes < 10 {
                        format!("0{}",minutes)
                    }
                    else {minutes.to_string()}
                };
                let seconds = (time::get_time()-timer_offset).floor() as usize % 60;
                let seconds = {
                    if seconds < 10 {
                        format!("0{}",seconds)
                    }
                    else {seconds.to_string()}
                };
                display_time = format!("{}:{}", minutes, seconds);
            };
            text::draw_text(&display_time, screen_width()*0.4, y_offset*0.8, y_offset, color::BLACK);
        }

        if game_over {
            shapes::draw_rectangle(screen_width()/5.0, screen_height()/5.0+y_offset, screen_width()*0.6, screen_height()*0.4, color::BROWN);
            let result_text = {
                if won {"YOU WIN!"}
                else {"YOU LOSE!"}
            };
            text::draw_text(result_text, screen_width()/3.1, screen_height()/2.9+y_offset, 90.0, color::BLACK);
            text::draw_text(&format!("TIME: {}", display_time), screen_width()/2.9, screen_height()/2.9+scale*5.0, 50.0, color::BLACK);
            text::draw_text("PRESS R TO RESTART", screen_width()/3.6, screen_height()/2.9+scale*7.0, 50.0, color::BLACK)
        }
        window::next_frame().await;
        // sleeps
        thread::sleep(Duration::from_millis(12));
    }
}


fn gen_bombs(mut bombs_left: u32, bomb_field: &mut [Option<Tile>]) {
    println!("Generating {} bombs!", bombs_left);
    let n_tiles = (GAME_HEIGHT*GAME_WIDTH) as u32;
    let mut r_range: Vec<u32> = (0..n_tiles).collect();
    rand::srand((time::get_time()*911540.0) as u64);
    let mut lcount = 0;
    while bombs_left > 0 {
        let b_index = {
            let rand_n = (rand::rand() % (n_tiles-lcount)) as usize;
            let tmp = r_range[rand_n];
            r_range.remove(rand_n);
            tmp as usize
        };
        lcount += 1;
        if bomb_field[b_index].is_none() {
            // so that bombs don't spawn at starting pos
            continue;
        }
        bombs_left -= 1;
        bomb_field[b_index].replace((true,false));
    };
}

fn surrounding_bombs(mut x:i32, mut y: i32, board: &[Option<Tile>]) -> i32 {
    if x < 0 || y < 0 || x >= GAME_WIDTH as i32 || y >= GAME_HEIGHT as i32 {
        return 9;
    }
    x -= 1; y -= 1;
    let mut b_count = 0;
    for ly in cmp::max(y, 0)..cmp::min(y+3, GAME_HEIGHT as i32) {
        for lx in cmp::max(x, 0)..cmp::min(x+3, GAME_WIDTH as i32) {
            if let Some((true,_)) = board[(ly as usize)*GAME_WIDTH + lx as usize] {
                b_count += 1;
            }
        }
    }
    b_count
}

fn bfs_destruction(x: i32, y: i32, board: &mut [Option<Tile>]) {
    board[(y as usize)*GAME_WIDTH+(x as usize)] = None;
    if surrounding_bombs(x, y, board) != 0 {
        return;
    }
    let surrounding_nodes = |ax: i32, ay: i32| {VecDeque::from([(ax+1,ay),(ax-1,ay),(ax,ay+1), (ax, ay-1)])};
    let mut nodes = VecDeque::from([(x,y)]);
    let mut visited: HashSet<(i32,i32)> = HashSet::new();
    while !nodes.is_empty() {
        let n = *nodes.front().unwrap();
        nodes.pop_front();
        if visited.contains(&n) {
            continue;
        }
        let n_bombs = surrounding_bombs(n.0, n.1, board);
        
        if n_bombs != 9 {
            board[(n.1 as usize)*GAME_WIDTH+n.0 as usize] = None;
        }
        if n_bombs == 0 {
            nodes.append(&mut surrounding_nodes(n.0,n.1 ));
        }
        visited.insert(n);
    }
}

fn have_won(board: &[Option<Tile>]) -> bool {
    let mut free_tiles = false;

    // works by combining all booleans together and simplifies with boolean logic ex: false false true false = true
    for t in board.iter().flatten() {
        if !t.0 {
            free_tiles = true;
        }
    }
    !free_tiles
}

fn reset_board(board: &mut Vec<Option<Tile>>) {
    for t in board {
        t.replace((false,false));
    }
}