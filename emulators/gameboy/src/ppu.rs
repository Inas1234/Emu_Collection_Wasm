use crate::memory::MemoryBus;
use crate::console_log;
pub struct GPU {
    vram: [u8; 0x2000],         // Video RAM
    oam: [u8; 0xA0],            // Object Attribute Memory (Sprites)
    lcd_control: u8,            // LCD Control (LCDC)
    lcd_status: u8,             // LCD Status (STAT)
    scroll_x: u8,               // Scroll X (SCX)
    scroll_y: u8,               // Scroll Y (SCY)
    window_x: u8,               // Window X (WX)
    window_y: u8,               // Window Y (WY)
    current_scanline: u8,       // Current Y line (LY)
    ly_compare: u8,             // LY Compare (LYC)
    background_palette: u8,     // Background Palette (BGP)
    sprite_palette_0: u8,       // Object Palette 0 (OBP0)
    sprite_palette_1: u8,       // Object Palette 1 (OBP1)
    frame_buffer: [[u8; 160]; 144], // Frame buffer to store pixel data
    mode_clock: u32,            // Clock for tracking mode timing
    mode: GPUMode,              // Current GPU mode (OAM, VRAM, HBlank, VBlank)
    bus: MemoryBus
}

#[derive(Clone, Copy, PartialEq)]
enum GPUMode {
    HBlank,
    VBlank,
    OAM,
    VRAM,
}

impl GPU {
    pub fn new(bus: MemoryBus) -> Self {
        Self {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcd_control: 0,
            lcd_status: 0,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            current_scanline: 0,
            ly_compare: 0,
            background_palette: 0,
            sprite_palette_0: 0,
            sprite_palette_1: 0,
            frame_buffer: [[0; 160]; 144],
            mode_clock: 0,
            mode: GPUMode::OAM,
            bus
        }
    }

    pub fn get_frame_buffer_ptr(&self) -> *const u8 {
        self.frame_buffer.as_ptr() as *const u8
    }

    /// Get the length of the frame buffer (width * height)
    pub fn get_frame_buffer_len(&self) -> usize {
        self.frame_buffer.len() * self.frame_buffer[0].len()
    }

    fn render_sprites(&mut self) {
        // Check if sprites are enabled in LCD control
        if self.lcd_control & 0x02 == 0 {
            return;
        }

        let sprite_height = if self.lcd_control & 0x04 != 0 { 16 } else { 8 };

        let mut sprites_rendered = 0;

        for i in (0..40).rev() {
            if sprites_rendered >= 10 {
                break;
            }

            let index = i * 4;
            let y_pos = self.oam[index] as i16 - 16;
            let x_pos = self.oam[index + 1] as i16 - 8;
            let tile_index = self.oam[index + 2];
            let attributes = self.oam[index + 3];

            let flip_x = attributes & 0x20 != 0;
            let flip_y = attributes & 0x40 != 0;
            let use_palette1 = attributes & 0x10 != 0;
            let priority = attributes & 0x80 != 0;

            if self.current_scanline < y_pos as u8 || self.current_scanline >= (y_pos + sprite_height) as u8 {
                continue;
            }

            let line = if flip_y {
                sprite_height - 1 - (self.current_scanline as i16 - y_pos)
            } else {
                self.current_scanline as i16 - y_pos
            };

            let tile_address = 0x8000 + (tile_index as u16 * 16) + (line as u16 * 2);
            let byte1 = self.vram[(tile_address - 0x8000) as usize];
            let byte2 = self.vram[(tile_address - 0x8000 + 1) as usize];

            for x in 0..8 {
                let color_bit = if flip_x { x } else { 7 - x };
                let color_id = ((byte1 >> color_bit) & 1) | (((byte2 >> color_bit) & 1) << 1);

                if color_id == 0 {
                    continue;
                }

                let palette = if use_palette1 { self.sprite_palette_1 } else { self.sprite_palette_0 };
                let color = self.get_color(color_id, palette);

                let pixel_x = x_pos + x;
                if pixel_x < 0 || pixel_x >= 160 {
                    continue;
                }

                if priority && self.frame_buffer[self.current_scanline as usize][pixel_x as usize] != 0xFF {
                    continue;
                }

                self.frame_buffer[self.current_scanline as usize][pixel_x as usize] = color;
            }

            sprites_rendered += 1;
        }
    }



    pub fn step(&mut self, cycles: u32) {
        self.mode_clock += cycles;
    
        match self.mode {
            GPUMode::OAM => {
                if self.mode_clock >= 80 {
                    self.mode_clock = 0;
                    self.mode = GPUMode::VRAM;
                }
            }
            GPUMode::VRAM => {
                if self.mode_clock >= 172 {
                    self.mode_clock = 0;
                    self.render_scanline();
                    self.render_sprites();
                    self.mode = GPUMode::HBlank;
                }
            }
            GPUMode::HBlank => {
                if self.mode_clock >= 204 {
                    self.mode_clock = 0;
                    self.current_scanline += 1;
    
                    if self.current_scanline == 144 {
                        self.mode = GPUMode::VBlank;
                        self.request_vblank_interrupt();
                    } else {
                        self.mode = GPUMode::OAM;
                    }
                }
            }
            GPUMode::VBlank => {
                if self.mode_clock >= 456 {
                    self.mode_clock = 0;
                    self.current_scanline += 1;
    
                    if self.current_scanline > 153 {
                        self.mode = GPUMode::OAM;
                        self.current_scanline = 0;
                    }
                }
            }
        }
    }
    
    fn render_scanline(&mut self) {
        if self.lcd_control & 0x80 == 0 {
            console_log!("LCD is disabled, skipping scanline rendering");
            return;
        }
    
        let tile_data = if self.lcd_control & 0x10 != 0 { 0x8000 } else { 0x8800 };
        let tile_map = if self.lcd_control & 0x08 != 0 { 0x9C00 } else { 0x9800 };
    
        console_log!("Rendering scanline: {}", self.current_scanline);
        console_log!("LCD Control: {:08b}", self.lcd_control);
    
        for x in 0..160 {
            let pixel = self.get_background_pixel(x, self.current_scanline, tile_data, tile_map);
            self.frame_buffer[self.current_scanline as usize][x as usize] = pixel;
    
            // Log if a non-zero pixel is written
            if pixel != 0 {
                console_log!("Non-zero pixel written at ({}, {}): {}", x, self.current_scanline, pixel);
            }
        }
    }
    
    fn get_background_pixel(&self, x: u8, y: u8, tile_data: u16, tile_map: u16) -> u8 {
        let map_offset = ((y as u16 / 8) * 32) + (x as u16 / 8);
        let tile_number = self.vram[(tile_map - 0x8000 + map_offset) as usize];
        let tile_address = if tile_data == 0x8000 {
            tile_data + (tile_number as u16 * 16)
        } else {
            (tile_data as i16 + (tile_number as i8 as i16) * 16) as u16
        };
    
        let line = y % 8;
        let byte1 = self.vram[(tile_address + (line as u16 * 2) + 1) as usize];
        let byte2 = self.vram[(tile_address + (line as u16 * 2) + 2) as usize];
    
        let color_bit = 7 - (x % 8);
        let color_id = ((byte1 >> color_bit) & 1) | (((byte2 >> color_bit) & 1) << 1);
    
        let color = self.get_color(color_id as u8, self.background_palette);
    
        if color != 0 {
            console_log!("Non-zero background color at ({}, {}): {}", x, y, color);
        }
    
        color
    }
    
    fn get_color(&self, color_id: u8, palette: u8) -> u8 {
        match (palette >> (color_id * 2)) & 0b11 {
            0 => 0xFF, 
            1 => 0xC0, 
            2 => 0x80, 
            3 => 0x00, 
            _ => 0xFF,
        }
    }

    fn request_vblank_interrupt(&mut self) {
        if self.bus.interrupt_enable & 0x01 != 0 {
            self.bus.interrupt_flag |= 0x01;
        }
        }

    fn request_lcd_interrupt(&mut self) {
        if self.bus.interrupt_enable & 0x01 != 0 {
            self.bus.interrupt_flag |= 0x01;
        }
        }
}
