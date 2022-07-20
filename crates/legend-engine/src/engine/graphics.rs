use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use byteorder::ReadBytesExt;

#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color {
    pub fn new(r: u8, g:u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn alpha_blend(&self, target: &Color, alpha: f64) -> Self {
        Color::new(
            ((self.r as f64) * (1.0 - alpha) + (target.r as f64) * alpha) as u8,
            ((self.g as f64) * (1.0 - alpha) + (target.g as f64) * alpha) as u8,
            ((self.b as f64) * (1.0 - alpha) + (target.b as f64) * alpha) as u8,
            255
        )
    }

    pub fn blend(&self, target: &Color) -> Self {
        let alpha = (target.a as f64) / 255.0;
        self.alpha_blend(target, alpha)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T
}

impl<T> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2 { x, y }
    }
}

pub struct Palette {
    colors: [Color; 256]
}

impl Palette {
    fn new(filename: &str) -> Option<Self> {
        if let Ok(mut file) = File::open(filename) {
            let mut buffer = [0; 256 * 3];
            if let Ok(_) = file.read(&mut buffer) {
                let mut colors = [Color::new(0, 0, 0, 255); 256];

                for (i, pixel) in buffer.chunks_exact_mut(4).enumerate() {
                    colors[i] = Color::new(pixel[0] * 4, pixel[1] * 4, pixel[2] * 4, 255);
                }

                return Some(Self { colors });
            }
        }

        None
    }

    pub fn create_by_buffer<R: Read>(buffer: &mut R) -> Self {
        let mut colors = [Color::new(0, 0, 0, 255); 256];

        for color in colors.iter_mut() {
            color.r = buffer.read_u8().unwrap_or(0);
            color.g = buffer.read_u8().unwrap_or(0);
            color.b = buffer.read_u8().unwrap_or(0);
        }

        Self { colors }
    }

    pub fn get_color(&self, index: u8) -> Color {
        self.colors[index as usize]
    }

    pub fn set_color(&mut self, index: u8, color: Color) {
        self.colors[index as usize] = color;
    }

    pub fn swap(&mut self, index_a: u8, index_b: u8) {
        let color_a = self.get_color(index_a);
        self.set_color(index_a, self.get_color(index_b));
        self.set_color(index_b, color_a);
    }

    pub fn animate(&mut self, index: u8, count: u8) {
        let color = self.get_color(index);
        for i in 0..count {
            self.set_color(index - i, self.get_color(index - i - 1));
        }
        self.set_color(index - count, color);
    }

    pub fn empty() -> Self {
        Self { colors: [Color::new(0, 0, 0, 255); 256] }
    }
}


pub struct Font {
    width: usize,
    height: usize,
    data: Vec<u8>
}

impl Font {
    fn new(filename: &str, width: usize, height: usize) -> Option<Self> {
        let mut data: Vec<u8> = Vec::new();
        let mut file = File::open(filename).unwrap();
        if let Ok(_) = file.read_to_end(&mut data) {
            Some(Self { width, height, data })
        } else {
            None
        }
    }
}

pub struct GameFont {
    english_font: Font,
    chinese_font: Font
}

impl GameFont {
    fn new(english_filename: &str, chinese_filename: &str) -> Option<Self> {
        if let Some(english_font) = Font::new(english_filename, 8, 16) {
            if let Some(chinese_font) = Font::new(chinese_filename, 16, 16) {
                return Some(Self { english_font, chinese_font });
            };
        };

        None
    }

    pub fn get_height(&self) -> i32 {
        max(self.english_font.height as i32, self.chinese_font.height as i32)
    }

    pub fn get_width(&self, text: &[usize]) -> i32 {
        let mut width = 0;

        for &character in text {
            let font = if character < 128 {
                &self.english_font
            } else {
                &self.chinese_font
            };

            width += font.width as i32;
        }

        width
    }
}

#[derive(Clone)]
pub struct RleImage {
    pub size: Vector2<u16>,
    pub offset: Vector2<i16>,
    data: Vec<u8>
}

impl RleImage {
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn reference_index(&self) -> usize {
        self.size.x as usize
    }
}

#[derive(Clone)]
pub struct Image {
    pub size: Vector2<u32>,
    pub data: Vec<Color>
}


fn blit<T, F>(target: &mut Vec<T>, width: i32, height: i32, source: &RleImage, x: i32, y: i32, value_function: F) where F: Fn(&RleImage, usize) -> T {
    let mut start_x = x + source.offset.x as i32;

    if start_x >= width {
        return;
    } else if (start_x + source.size.x as i32) <= 0 {
        return;
    }

    let mut start_y = y + source.offset.y as i32;

    if start_y >= height {
        return;
    } else if (start_y + source.size.y as i32) <= 0 {
        return;
    }

    let mut index: usize = 0;

    for j in 0..source.size.y as i32 {
        let line_start_index = index;
        let line_length = source.data[index];
        index += 1;

        let mut current_x = start_x;

        while index - line_start_index < line_length as usize {
            current_x += source.data[index] as i32;
            index += 1;

            if index - line_start_index >= line_length as usize {
                break;
            }

            let data_length = source.data[index] as i32;
            index += 1;

            for _ in 0..data_length {
                let current_y = start_y + j;

                if current_x >= 0 && current_x < width && current_y >= 0 && current_y < height {
                    target[(current_y * width + current_x) as usize] = value_function(source, index);
                }

                index += 1;
                current_x += 1;
            }
        }
    }
}

impl Image {

    pub fn blit(&mut self, source: &RleImage, x: i32, y: i32, palette: &Palette) {
        blit(&mut self.data, self.size.x as i32, self.size.y as i32, source, x, y, |rle_image, index| { palette.get_color(rle_image.data[index]) });
    }

    pub fn alpha_blit(&mut self, source: &Image, x: i32, y: i32, alpha: f64) {
        for j in 0..source.size.y as i32 {
            if y + j < 0 || y + j >= self.size.y as i32 {
                continue;
            }
            for i in 0..source.size.x as i32 {
                if x + i < 0 || x + i >= self.size.x as i32 {
                    continue;
                }

                let source_color = source.data[j as usize * source.size.x as usize + i as usize].clone();

                if source_color.a == 0 {
                    continue;
                }

                let pixel = {
                    let dest_color = &self.data[(j + y) as usize * self.size.x as usize + (x + i) as usize];
                    dest_color.alpha_blend(&source_color, alpha)
                };

                self.set_pixel(i + x, j + y, &pixel);
            }
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: &Color) {
        for j in 0..height {
            if y + j < 0 || y + j >= self.size.y as i32 {
                continue;
            }

            for i in 0..width {
                if x + i < 0 || x + i >= self.size.x as i32 {
                    continue;
                }

                let pixel = {
                    let dest_color = &self.data[(j + y) as usize * self.size.x as usize + (x + i) as usize];
                    dest_color.blend(color)
                };

                self.set_pixel(i + x, j + y, &pixel);
            }
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: &Color) {
        if x < 0 || x >= self.size.x as i32 {
            return;
        }

        if y < 0 || y >= self.size.y as i32 {
            return;
        }

        let index = y as usize * self.size.x as usize + x as usize;

        self.data[index] = color.clone();
    }

    pub fn draw_char(&mut self, character: usize, x: i32, y: i32, font: &Font, color: &Color) {
        let character_bytes = (font.width / 8) * font.height;

        // big5 page
        let code = if character >= 0xa140 {
            let page = (character & 0xff00) / 0x100 - 0xa1;

            let position = if (character & 0xff) >= 0xa1 {
                (character & 0xff) - 0xa1 + 0x7e - 0x40 + 1
            } else {
                (character & 0xff) - 0x40
            };

            page * (0xfe - 0xa1 + 0x7e - 0x40 + 2) + position
        } else {
            character
        };

        let index = character_bytes * code;

        // out of bound
        if index + character_bytes > font.data.len() {
            return;
        }

        for j in 0..font.height {
            let mut current_x = 0;
            for i in 0..(font.width / 8) {
                let byte = font.data[index + j * (font.width / 8) + i];

                for bit in (0..8).rev() {
                    if (1 << bit) & byte > 0 {
                        self.set_pixel(x + current_x, y + j as i32, color);
                    }

                    current_x += 1;
                }
            }
        }
    }

    pub fn draw_text(&mut self, text: &[usize], x: i32, y: i32, font: &Font, color: &Color) {
        let mut count = 0;
        let mut line = 0;
        for &character in text {
            if character == 13 {
                line += 1;
                count = 0;
            }

            self.draw_char(character, x + count * font.width as i32, y + line * font.height as i32, font, color);
            count += 1;
        }
    }

    pub fn draw_game_text(&mut self, text: &[usize], x: i32, y: i32, game_font: &GameFont, color: &Color) {

        let mut offset = 0;
        for &character in text {
            let font = if character < 128 {
                &game_font.english_font
            } else {
                &game_font.chinese_font
            };

            self.draw_char(character, x + offset, y, font, color);
            offset += font.width as i32;
        }
    }

    pub fn draw_game_text_center(&mut self, text: &[usize], x: i32, y: i32, width: i32, height: i32, game_font: &GameFont, color: &Color) {
        let text_width = game_font.get_width(text);
        self.draw_game_text(text, x + (width - text_width) / 2, y + (height - game_font.get_height()) / 2, game_font, color);
    }

    pub fn draw_shadow_text(&mut self, text: &[usize], x: i32, y: i32, game_font: &GameFont, color: &Color, shadow_color: &Color) {
        self.draw_game_text(text, x + 1, y, game_font, shadow_color);
        self.draw_game_text(text, x, y, game_font, color);
    }

    pub fn draw_shadow_text_center(&mut self, text: &[usize], x: i32, y: i32, width: i32, height: i32, game_font: &GameFont, color: &Color, shadow_color: &Color) {
        let text_width = game_font.get_width(text);
        self.draw_shadow_text(text, x + (width - text_width) / 2, y + (height - game_font.get_height()) / 2, game_font, color, shadow_color);
    }

    pub fn copy_to(&self, buffer: &mut [u8]) {
        let mut i: usize = 0;

        for color in &self.data {
            buffer[i] = color.r;
            buffer[i + 1] = color.g;
            buffer[i + 2] = color.b;
            buffer[i + 3] = color.a;
            i += 4;
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0; (self.size.x * self.size.y * 4) as usize];
        self.copy_to(&mut buffer);
        buffer
    }

    pub fn new(width: u32, height: u32) -> Image {
        Image {
            size: Vector2::new(width, height),
            data: vec![Color::new(0, 0, 0, 0); (width * height) as usize]
        }
    }

    pub fn clear_by_color(&mut self, color: Color) {
        for pixel in self.data.iter_mut() {
            *pixel = color;
        }
    }

    pub fn clear(&mut self) {
        self.clear_by_color(Color::new(0, 0, 0, 0));
    }

    pub fn save(&self, filename: &str) {
        let mut image_to_save: image::RgbaImage = image::ImageBuffer::new(self.size.x, self.size.y);

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let color = self.data.get((y * self.size.x + x) as usize).unwrap();
                image_to_save.put_pixel(x, y, image::Rgba([color.r, color.g, color.b, color.a]));
            }
        }

        image_to_save.save(filename).unwrap();
    }
}

pub struct Graphics {
    frame_buffer: Image,
    effect_buffers: HashMap<String, Image>,
    width: u32,
    height: u32
}

impl Graphics {
    pub fn new(width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            frame_buffer: Image::new(width, height),
            effect_buffers: HashMap::new(),
            width,
            height
        })
    }

    pub fn render_to(&self, frame_buffer: &mut [u8]) -> Result<(), Box<dyn Error>> {

        self.frame_buffer.copy_to(frame_buffer);

        Ok(())
    }
}