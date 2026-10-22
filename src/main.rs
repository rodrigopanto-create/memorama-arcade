use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
mod highscores;
pub use highscores::{HighScore, HighScores};

fn game_width() -> f32 {
    if cfg!(target_arch = "wasm32") {
        if screen_height() > screen_width() { 400.0 } else { 800.0 }
    } else { screen_width() }
}

fn game_height() -> f32 {
    if cfg!(target_arch = "wasm32") {
        game_width() * screen_height() / screen_width()
    } else { screen_height() }
}

fn game_mouse_position() -> (f32, f32) {
    let (x, y) = mouse_position();
    (x * game_width() / screen_width(), y * game_height() / screen_height())
}

fn configure_game_camera() {
    #[cfg(target_arch = "wasm32")]
    set_camera(&Camera2D {
        target: vec2(game_width() / 2.0, game_height() / 2.0),
        zoom: vec2(2.0 / game_width(), 2.0 / game_height()),
        ..Default::default()
    });
}

async fn next_game_frame() {
    next_frame().await;
    configure_game_camera();
}

// Estados posibles de una carta
#[derive(Clone, Copy, PartialEq)]
enum CardState {
    Hidden,
    Flipped,
    Matched,
}

// Niveles de dificultad
#[derive(Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,   // 3x4 (12 cartas, 6 parejas)
    Normal, // 4x4 (16 cartas, 8 parejas)
    Hard,   // 4x6 (24 cartas, 12 parejas)
}

struct Card {
    value: usize,
    state: CardState,
}

// Helper para dibujar texto perfectamente centrado horizontalmente
fn draw_text_centered(text: &str, y: f32, font_size: f32, color: Color) {
    let width = measure_text(text, None, font_size as u16, 1.0).width;
    let font_size = font_size * ((game_width() - 32.0) / width.max(1.0)).min(1.0);
    let text_dim = measure_text(text, None, font_size as u16, 1.0);
    let x = (game_width() - text_dim.width) / 2.0;
    draw_text(text, x, y, font_size, color);
}

fn draw_button(rect: Rect, label: &str, selected: bool) -> bool {
    let (x, y) = game_mouse_position();
    let hovered = rect.contains(vec2(x, y));
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, if selected || hovered { DARKGREEN } else { DARKBLUE });
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, WHITE);
    let size = 24.0;
    let text = measure_text(label, None, size as u16, 1.0);
    draw_text(label, rect.x + (rect.w - text.width) / 2.0, rect.y + (rect.h + text.height) / 2.0, size, WHITE);
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

// --- MÓDULO DE AUDIO RETRO (Stubs) ---
fn play_sound_effect(effect: &str) {
    match effect {
        "flip" => {}
        "match" => {}
        "fail" => {}
        "win" => {}
        _ => {}
    }
}

/// Pantalla de Inicio Estilo Atari / Arcade (Soporta Clic, Enter y Espacio)
async fn run_title_screen() {
    let mut timer: f32 = 0.0;
    loop {
        let delta = get_frame_time();
        timer += delta;
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));

        let sh = game_height();

        let progress = (timer / 0.9).clamp(0.0, 1.0);
        let eased_progress = 1.0 - (1.0 - progress).powi(3);
        let title_y = -50.0 + (sh / 2.0 - 10.0) * eased_progress;
        draw_text_centered("MEMORAMA", title_y, 50.0, Color::new(1.0, 0.8, 0.0, 1.0));
        draw_text_centered("ARCADE SUITE V0.3", sh / 2.0 - 10.0, 20.0, SKYBLUE);

        let prompt_alpha = if (timer * 6.0).sin() > 0.0 { 1.0 } else { 0.3 };
        draw_text_centered("PRESS START", sh / 2.0 + 60.0, 18.0, Color::new(0.0, 1.0, 0.5, prompt_alpha));

        draw_text_centered("© 2026 RETRO LABS", sh - 40.0, 14.0, GRAY);

        // Activación por Clic, Enter o Espacio
        if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            play_sound_effect("flip");
            next_game_frame().await;
            break;
        }

        next_game_frame().await;
    }
}

/// Menú de Selección de Dificultad
async fn run_difficulty_menu() -> Difficulty {
    loop {
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));

        let sw = game_width();
        let sh = game_height();

        draw_text_centered("SELECCIONA DIFICULTAD", sh / 2.0 - 100.0, 24.0, WHITE);

        let btn_w = 240.0;
        let btn_h = 45.0;
        let start_y = sh / 2.0 - 40.0;

        let options = vec![
            ("FACIL (3x4)", Difficulty::Easy, start_y),
            ("NORMAL (4x4)", Difficulty::Normal, start_y + 60.0),
            ("EXPERTO (4x6)", Difficulty::Hard, start_y + 120.0),
        ];

        let mut chosen = None;

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = game_mouse_position();
            for (_, diff, y) in &options {
                let x = (sw - btn_w) / 2.0;
                if mx >= x && mx <= x + btn_w && my >= *y && my <= *y + btn_h {
                    chosen = Some(*diff);
                }
            }
        }

        for (label, _diff, y) in &options {
            let x = (sw - btn_w) / 2.0;
            let (mx, my) = game_mouse_position();
            let hovered = mx >= x && mx <= x + btn_w && my >= *y && my <= *y + btn_h;

            let bg_col = if hovered { DARKGREEN } else { DARKBLUE };
            draw_rectangle(x, *y, btn_w, btn_h, bg_col);
            draw_rectangle_lines(x, *y, btn_w, btn_h, 2.0, WHITE);
            
            let text_dim = measure_text(label, None, 20, 1.0);
            let text_x = x + (btn_w - text_dim.width) / 2.0;
            draw_text(label, text_x, *y + 28.0, 20.0, WHITE);
        }

        if let Some(diff) = chosen {
            next_game_frame().await;
            return diff;
        }

        next_game_frame().await;
    }
}

/// Pantalla de High Scores Arcade (Navegación completa con flechas y confirmación)
async fn run_highscore_screen(final_score: i32, high_scores: &mut HighScores) {
    let qualifies = high_scores.entries.len() < 10 || final_score > high_scores.entries.iter().map(|h| h.score).min().unwrap_or(0);
    let mut save_error = None;

    if qualifies {
        let mut initials = [b'A', b'A', b'A'];
        let mut current_char: usize = 0;

        loop {
            clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
            let sh = game_height();

            draw_text_centered("¡NUEVO RECORD!", sh / 2.0 - 100.0, 28.0, YELLOW);
            
            let score_msg = format!("PUNTAJE: {}", final_score);
            draw_text_centered(&score_msg, sh / 2.0 - 60.0, 22.0, WHITE);
            draw_text_centered("INGRESA TUS INICIALES:", sh / 2.0 - 20.0, 18.0, LIGHTGRAY);

            for (i, &ch) in initials.iter().enumerate() {
                let rect = Rect::new(game_width() / 2.0 - 112.0 + i as f32 * 80.0, sh / 2.0, 64.0, 60.0);
                if draw_button(rect, &(ch as char).to_string(), i == current_char) {
                    current_char = i;
                }
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = game_mouse_position();
                let rect = Rect::new(game_width() / 2.0 - 112.0 + current_char as f32 * 80.0, sh / 2.0, 64.0, 60.0);
                if rect.contains(vec2(x, y)) {
                    initials[current_char] = if initials[current_char] == b'Z' { b'A' } else { initials[current_char] + 1 };
                }
            }
            draw_text_centered("TOCA CADA LETRA PARA CAMBIARLA", sh / 2.0 + 90.0, 16.0, GRAY);
            let save_clicked = draw_button(
                Rect::new(game_width() / 2.0 - 100.0, sh / 2.0 + 115.0, 200.0, 52.0),
                "GUARDAR", false,
            );

            // Cambiar valor de la letra actual (Arriba / Abajo)
            if is_key_pressed(KeyCode::Up) {
                initials[current_char] = if initials[current_char] == b'Z' { b'A' } else { initials[current_char] + 1 };
            }
            if is_key_pressed(KeyCode::Down) {
                initials[current_char] = if initials[current_char] == b'A' { b'Z' } else { initials[current_char] - 1 };
            }

            // Moverse entre letras (Izquierda / Derecha)
            if is_key_pressed(KeyCode::Right) {
                if current_char < 2 {
                    current_char += 1;
                }
            }
            if is_key_pressed(KeyCode::Left) {
                if current_char > 0 {
                    current_char -= 1;
                }
            }

            // Confirmar y guardar (Enter o Espacio solo si estás en la última letra o deseas registrar)
            if save_clicked || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                if !save_clicked && current_char < 2 {
                    current_char += 1; // Salta a la siguiente letra por comodidad
                } else {
                    let name = String::from_utf8(initials.to_vec()).unwrap_or_else(|_| "AAA".to_string());
                    high_scores.entries.push(HighScore { name, score: final_score });
                    high_scores.entries.sort_by(|a, b| b.score.cmp(&a.score));
                    high_scores.entries.truncate(10);
                    if let Err(err) = high_scores.save() {
                        error!("{}", err);
                        save_error = Some(err);
                    }
                    next_game_frame().await;
                    break;
                }
            }

            next_game_frame().await;
        }
    }

    let mut waiting = true;
    while waiting {
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
        let sh = game_height();

        draw_text_centered("TABLA DE MEJORES PUNTAJES", 60.0, 24.0, YELLOW);

        let mut y_offset = 100.0;
        let row_height = ((sh - 190.0) / 10.0).clamp(16.0, 30.0);
        for (i, hs) in high_scores.entries.iter().enumerate() {
            let entry = format!("{:2}.  {}  ---  {} PTS", i + 1, hs.name, hs.score);
            draw_text_centered(&entry, y_offset, 18.0, WHITE);
            y_offset += row_height;
        }

        if save_error.is_some() {
            draw_text_centered("ERROR AL GUARDAR: RECORD SOLO EN ESTA SESION", sh - 80.0, 16.0, RED);
        }
        draw_text_centered("CLIC PARA VOLVER AL MENU", sh - 50.0, 16.0, LIGHTGRAY);

        if is_mouse_button_pressed(MouseButton::Left) {
            waiting = false;
        }

        next_game_frame().await;
    }
}

pub async fn run_memorama(difficulty: Difficulty, high_scores: &mut HighScores) -> bool {
    let mut load_timer: f32 = 0.0;
    while load_timer < 0.6 {
        load_timer += get_frame_time();
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
        draw_text_centered("CARGANDO...", game_height() / 2.0, 20.0, GREEN);
        next_game_frame().await;
    }

    let total_cards = match difficulty {
        Difficulty::Easy => 12,
        Difficulty::Normal => 16,
        Difficulty::Hard => 24,
    };

    let mut values: Vec<usize> = (0..total_cards / 2).chain(0..total_cards / 2).collect();
    values.shuffle();

    let mut cards: Vec<Card> = values
        .into_iter()
        .map(|value| Card {
            value,
            state: CardState::Hidden,
        })
        .collect();

    let mut flipped_indices: Vec<usize> = Vec::new();
    let mut lock_timer: f32 = 0.0;
    let mut victory_timer: f32 = 0.0;

    let mut attempts: usize = 0;
    let mut score: i32 = 1000;
    let mut combo: usize = 0;
    let mut elapsed_time: f32 = 0.0;
    let mut calculated_final_score: Option<i32> = None;

    loop {
        let delta = get_frame_time();
        clear_background(Color::new(0.1, 0.1, 0.15, 1.0));

        let won = cards.iter().all(|c| c.state == CardState::Matched);

        if won {
            if calculated_final_score.is_none() {
                play_sound_effect("win");
                let final_pts = (score as f32 - (elapsed_time * 2.0)).max(100.0) as i32;
                calculated_final_score = Some(final_pts);
            }

            victory_timer += delta;
            let final_pts = calculated_final_score.unwrap();

            let sh = game_height();
            draw_text_centered("¡VICTORIA ARCADE!", sh / 2.0 - 30.0, 32.0, GREEN);
            let score_msg = format!("PUNTAJE FINAL: {}", final_pts);
            draw_text_centered(&score_msg, sh / 2.0 + 15.0, 22.0, WHITE);

            if victory_timer > 2.5 {
                run_highscore_screen(final_pts, high_scores).await;
                return true;
            }
            next_game_frame().await;
            continue;
        }

        elapsed_time += delta;

        if lock_timer > 0.0 {
            lock_timer -= delta;
            if lock_timer <= 0.0 {
                for &idx in &flipped_indices {
                    cards[idx].state = CardState::Hidden;
                }
                flipped_indices.clear();
            }
        }

        let portrait = game_height() > game_width();
        let (grid_cols, grid_rows) = grid_dimensions(difficulty, portrait);
        let gap = 10.0;
        let card_size = if cfg!(target_arch = "wasm32") {
            ((game_width() - 32.0 - gap * (grid_cols - 1) as f32) / grid_cols as f32)
                .min((game_height() - 160.0 - gap * (grid_rows - 1) as f32) / grid_rows as f32)
                .clamp(20.0, 120.0)
        } else { match difficulty {
            Difficulty::Hard => 60.0,
            _ => 75.0,
        }};
        let total_w = grid_cols as f32 * (card_size + gap) - gap;
        let total_h = grid_rows as f32 * (card_size + gap) - gap;
        let start_x = (game_width() - total_w) / 2.0;
        let start_y = (game_height() - total_h) / 2.0 + 20.0;

        if lock_timer <= 0.0 && is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = game_mouse_position();

            for i in 0..cards.len() {
                if cards[i].state != CardState::Hidden {
                    continue;
                }

                let col = i % grid_cols;
                let row = i / grid_cols;
                let x = start_x + col as f32 * (card_size + gap);
                let y = start_y + row as f32 * (card_size + gap);

                if mouse_x >= x && mouse_x <= x + card_size && mouse_y >= y && mouse_y <= y + card_size {
                    cards[i].state = CardState::Flipped;
                    flipped_indices.push(i);
                    play_sound_effect("flip");

                    if flipped_indices.len() == 2 {
                        attempts += 1;
                        let idx1 = flipped_indices[0];
                        let idx2 = flipped_indices[1];

                        if cards[idx1].value == cards[idx2].value {
                            cards[idx1].state = CardState::Matched;
                            cards[idx2].state = CardState::Matched;
                            flipped_indices.clear();
                            combo += 1;
                            score += 150 * (combo as i32);
                            play_sound_effect("match");
                        } else {
                            lock_timer = 0.8;
                            combo = 0;
                            score = (score - 50).max(0);
                            play_sound_effect("fail");
                        }
                    }
                    break;
                }
            }
        }

        draw_text_centered("MEMORAMA", start_y - 75.0, 24.0, WHITE);
        draw_text_centered(&format!("TIEMPO: {:.1}s  |  INTENTOS: {}", elapsed_time, attempts), start_y - 48.0, 18.0, LIGHTGRAY);
        draw_text_centered(&format!("COMBO: x{}  |  PTS: {}", combo, score), start_y - 22.0, 18.0, LIGHTGRAY);

        for i in 0..cards.len() {
            let col = i % grid_cols;
            let row = i / grid_cols;
            let x = start_x + col as f32 * (card_size + gap);
            let y = start_y + row as f32 * (card_size + gap);

            let (bg_color, text, text_color) = match cards[i].state {
                CardState::Hidden => (DARKBLUE, "?".to_string(), LIGHTGRAY),
                CardState::Flipped => (WHITE, cards[i].value.to_string(), BLACK),
                CardState::Matched => (DARKGREEN, cards[i].value.to_string(), WHITE),
            };

            draw_rectangle(x, y, card_size, card_size, bg_color);
            draw_rectangle_lines(x, y, card_size, card_size, 2.0, BLACK);

            let font_size = (card_size * 0.5).min(48.0);
            let text_size = measure_text(&text, None, font_size as u16, 1.0);
            draw_text(&text, x + (card_size - text_size.width) / 2.0, y + (card_size + text_size.height) / 2.0, font_size, text_color);
        }

        next_game_frame().await
    }
}

fn grid_dimensions(difficulty: Difficulty, portrait: bool) -> (usize, usize) {
    match (difficulty, portrait) {
        (Difficulty::Easy, true) => (3, 4),
        (Difficulty::Easy, false) => (4, 3),
        (Difficulty::Normal, _) => (4, 4),
        (Difficulty::Hard, true) => (4, 6),
        (Difficulty::Hard, false) => (6, 4),
    }
}

#[test]
fn portrait_grids_preserve_pairs() {
    for (difficulty, count) in [(Difficulty::Easy, 12), (Difficulty::Normal, 16), (Difficulty::Hard, 24)] {
        for portrait in [false, true] {
            let (cols, rows) = grid_dimensions(difficulty, portrait);
            assert_eq!(cols * rows, count);
            if portrait { assert!(rows >= cols); }
        }
    }
}

#[macroquad::main("Memorama Minimalista - Arcade Suite")]
async fn main() {
    macroquad::rand::srand(macroquad::miniquad::date::now().to_bits());
    configure_game_camera();

    let mut high_scores = match HighScores::load() {
        Ok(scores) => scores,
        Err(err) => {
            error!("{}", err);
            loop {
                clear_background(BLACK);
                draw_text_centered("ERROR AL CARGAR LOS RECORDS", game_height() / 2.0 - 40.0, 24.0, RED);
                draw_text_centered("Se conservan los datos originales. Guardado desactivado.", game_height() / 2.0, 18.0, WHITE);
                draw_text_centered("CLIC O ENTER: JUGAR SIN GUARDAR", game_height() / 2.0 + 40.0, 18.0, YELLOW);
                if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter) {
                    break;
                }
                next_game_frame().await;
            }
            HighScores::defaults(false)
        }
    };

    loop {
        // 0. Pantalla de inicio Atari/Arcade
        run_title_screen().await;

        // 1. Menú de dificultad
        let chosen_difficulty = run_difficulty_menu().await;

        // 2. Juego principal
        let _ = run_memorama(chosen_difficulty, &mut high_scores).await;
    }
}