extern crate rand;
extern crate rustty;
extern crate termion;
extern crate tui;

use std::collections::HashSet;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rand::Rng;

use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{border, Block, Dataset, Marker, Widget, Chart, Axis};
use tui::widgets::canvas::Canvas;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style, Modifier};

type Cell = (usize, usize);
type CellSet = HashSet<Cell>;

const ALIVE: &'static str = "\u{25AA}";
const DEAD: &'static str = " ";

enum Event {
    Input(event::Key),
    Tick,
}


struct World {
    height: usize,
    width: usize,
    grid: CellSet,
}

impl World {
    fn new((width, height): Cell) -> World {
        World {
            height: height,
            width: width,
            grid: HashSet::with_capacity(height * width),
        }
    }
    fn gen(&mut self) {
        self.grid.clear();
        for x in 1..self.width {
            for y in 1..self.height {
                if rand::thread_rng().gen_weighted_bool(30) {
                    self.grid.insert((x, y));
                }
            }
        }
    }

    // This is an obviously dumb way to do this
    // TODO: Find a better way
    fn neighbors(&self, cell: &Cell) -> CellSet {
        let mut neighbors: CellSet = HashSet::with_capacity(8);
        let (x, y) = (cell.0, cell.1);

        let top = y.checked_sub(1) != None;
        let bot = y.checked_add(1) <= Some(self.height);
        let right = x.checked_add(1) <= Some(self.width);
        let left = x.checked_sub(1) != None;

        if right {
            neighbors.insert((x + 1, y));
        }
        if right && bot {
            neighbors.insert((x + 1, y + 1));
        }
        if right && top {
            neighbors.insert((x + 1, y - 1));
        }
        if bot {
            neighbors.insert((x, y + 1));
        }
        if top {
            neighbors.insert((x, y - 1));
        }
        if left {
            neighbors.insert((x - 1, y));
        }
        if left && bot {
            neighbors.insert((x - 1, y + 1));
        }
        if left && top {
            neighbors.insert((x - 1, y - 1));
        }
        neighbors
    }

    // TODO: Fix dumbness
    fn neighbor_count(&self, cell: &Cell) -> (CellSet, CellSet) {
        let mut neighbors: (CellSet, CellSet) =
            (HashSet::with_capacity(8), HashSet::with_capacity(8));
        for neighbor in self.neighbors(cell) {
            if self.grid.contains(&neighbor) {
                neighbors.0.insert(neighbor);
            } else {
                neighbors.1.insert(neighbor);
            }
        }
        neighbors
    }
    // TODO: undumb
    fn step(&mut self) {
        let mut new_state: CellSet = HashSet::with_capacity(self.width * self.height);

        for cell in &self.grid {
            let (living, dead) = self.neighbor_count(cell);
            if living.len() < 2 || living.len() > 3 {
            } else if living.len() == 2 || living.len() == 3 {
                new_state.insert(*cell);
            }

            for neighbor in dead {
                if self.neighbor_count(cell).0.len() == 3 {
                    new_state.insert(neighbor);
                }
            }
        }
        self.grid = new_state;
    }

    fn render(&self, t: &mut Terminal<MouseBackend>) {
        let tsize = t.size().unwrap();
        Chart::default()
            .block(Block::default().title("Chart"))
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .title_style(Style::default().fg(Color::Red))
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 10.0])
                    .labels(&[" "]),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .title_style(Style::default().fg(Color::Red))
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 10.0])
                    .labels(&[" "]),
            )
            .datasets(
                &[
                    Dataset::default()
                        .name("data1")
                        .marker(Marker::Dot)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
                    Dataset::default()
                        .name("data2")
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::Magenta))
                        .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
                ],
            )
            .render(t, &tsize);


        t.draw().unwrap();
    }
}


fn main() {
    //Create terminal and canvas
    let backend = MouseBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();

    // Channels
    // TODO: Understand this
    let (tx, rx) = mpsc::channel();
    let input_tx = tx.clone();
    let clock_tx = tx.clone();

    // Input
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            let evt = c.unwrap();
            input_tx.send(Event::Input(evt)).unwrap();
            if evt == event::Key::Char('q') {
                break;
            }
        }
    });

    // Tick
    thread::spawn(move || loop {
        clock_tx.send(Event::Tick).unwrap();
        thread::sleep(Duration::from_millis(500));
    });

    // First draw call
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    let tsize: (usize, usize) = (
        terminal.size().unwrap().height as usize,
        terminal.size().unwrap().width as usize,
    );
    let mut w = World::new(tsize);
    w.gen();
    w.render(&mut terminal);

    let mut auto = false;

    loop {
        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(input) => {
                match input {
                    event::Key::Char('q') => {
                        break;
                    }
                    event::Key::Char('g') => {
                        w.gen();
                    }
                    event::Key::Char('n') => {
                        w.step();
                    }
                    event::Key::Char('a') => {
                        auto = true;
                    }
                    event::Key::Char('s') => {
                        auto = false;
                    }
                    _ => {}
                }
            }
            Event::Tick => {
                if auto {
                    w.step();
                }
            }
        }
        //w.render(&mut terminal);
    }
}
