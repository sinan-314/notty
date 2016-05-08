//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::ops::Range;

use notty::cfg::Config;
use notty::terminal::Styles;

use cairo;
use cairo::glib::translate::ToGlibPtr;

use pangocairo::wrap::{PangoAttribute, PangoAttrList, PangoLayout};

use color;

pub struct TextRenderer<'a> {
    config: &'a Config,
    text: String,
    fg_color: Vec<(Range<usize>, (u8, u8, u8))>,
    bg_color: Vec<(Range<usize>, (u8, u8, u8))>,
    opacity: Vec<(Range<usize>, u8)>,
    underline: Vec<Range<usize>>,
    double_underline: Vec<Range<usize>>,
    bold: Vec<Range<usize>>,
    italic: Vec<Range<usize>>,
    strikethrough: Vec<Range<usize>>,
    blink: Vec<Range<usize>>,
    x_pos: f64,
    y_pos: f64,
}

impl<'a> TextRenderer<'a> {

    pub fn new(config: &'a Config, x: f64, y: f64) -> TextRenderer<'a> {
        TextRenderer {
            config: config,
            text: String::new(),
            fg_color: Vec::new(),
            bg_color: Vec::new(),
            opacity: Vec::new(),
            underline: Vec::new(),
            double_underline: Vec::new(),
            bold: Vec::new(),
            italic: Vec::new(),
            strikethrough: Vec::new(),
            blink: Vec::new(),
            x_pos: x,
            y_pos: y,
        }
    }

    pub fn push(&mut self, ch: char, styles: Styles) {
        let lower = self.text.len();
        self.text.push(ch);
        let range = lower..self.text.len();
        self.add_style(&range, styles);
    }

    pub fn push_str(&mut self, s: &str, styles: Styles) {
        let lower = self.text.len();
        self.text.push_str(s);
        let range = lower..self.text.len();
        self.add_style(&range, styles);
    }

    pub fn push_cursor(&mut self, ch: char, styles: Styles, cursor_styles: Styles) {
        let lower = self.text.len();
        self.text.push(ch);
        let range = lower..self.text.len();
        self.add_cursor_style(&range, styles, cursor_styles);
    }

    pub fn push_str_cursor(&mut self, s: &str, styles: Styles, cursor_styles: Styles) {
        let lower = self.text.len();
        self.text.push_str(s);
        let range = lower..self.text.len();
        self.add_cursor_style(&range, styles, cursor_styles);
    }

   pub fn draw(&self, canvas: &cairo::Context) {
        if self.is_blank() { return; }

        // Line positioning
        canvas.move_to(self.x_pos, self.y_pos);

        // Set text color
        let (r, g, b) = self.config.bg_color;
        canvas.set_source_rgb(color(r), color(g), color(b));

        // Draw the text
        let cairo = canvas.to_glib_none();
        PangoLayout::new(cairo.0, &self.config.font, &self.text, self.pango_attrs()).show(cairo.0);
    }

    fn is_blank(&self) -> bool {
        self.text.chars().all(char::is_whitespace)
        && self.bg_color.iter().all(|&(_, color)| color == self.config.bg_color)
    }

    fn add_style(&mut self, range: &Range<usize>, style: Styles) {
        let fg_color = self.config.fg_color(style.fg_color);
        let bg_color = self.config.bg_color(style.bg_color);
        if !style.inverted {
            append_field(range.clone(), fg_color, &mut self.fg_color);
            append_field(range.clone(), bg_color, &mut self.bg_color);
        } else {
            append_field(range.clone(), bg_color, &mut self.fg_color);
            append_field(range.clone(), fg_color, &mut self.bg_color);
        }
        append_field(range.clone(), style.opacity, &mut self.opacity);
        if style.underline { append_bool(range.clone(), &mut self.underline) }
        if style.double_underline { append_bool(range.clone(), &mut self.double_underline) }
        if style.bold { append_bool(range.clone(), &mut self.bold); }
        if style.italic { append_bool(range.clone(), &mut self.italic) }
        if style.strikethrough { append_bool(range.clone(), &mut self.strikethrough); }
        if style.blink { append_bool(range.clone(), &mut self.blink) }
    }

    fn add_cursor_style(&mut self, range: &Range<usize>, style: Styles, _: Styles) {
        self.add_style(range, Styles { inverted: !style.inverted, ..style });
    }

    fn pango_attrs(&self) -> PangoAttrList {
        self.fg_color.iter().map(|&(ref range, (r,g,b))| {
            PangoAttribute::fg_color(range, (r,g,b))
        }).chain(self.bg_color.iter().map(|&(ref range, (r,g,b))| {
            PangoAttribute::bg_color(range, (r,g,b))
        })).chain(self.underline.iter().map(PangoAttribute::underline))
        .chain(self.double_underline.iter().map(PangoAttribute::double_underline))
        .chain(self.bold.iter().map(PangoAttribute::bold))
        .chain(self.italic.iter().map(PangoAttribute::italic))
        .chain(self.strikethrough.iter().map(PangoAttribute::strikethrough))
        .collect()
    }

}

fn append_bool(range: Range<usize>, ranges: &mut Vec<Range<usize>>) {
    if let Some(last_range) = ranges.last_mut() {
        if last_range.end == range.start {
            return last_range.end = range.end;
        }
    }
    ranges.push(range);
}

fn append_field<T>(range: Range<usize>, field: T, ranges: &mut Vec<(Range<usize>, T)>)
where T: PartialEq + Copy {
    if let Some(&mut (ref mut last_range, last_field)) = ranges.last_mut() {
        if last_field == field {
            last_range.end = range.end;
            return;
        }
    }
    ranges.push((range, field));
}
