//Piston 2d snake game
extern crate glutin_window;
extern crate graphics;//This is piston 2d graphics
extern crate opengl_graphics;//This is the opengl graphics
extern crate piston;

use piston::window::WindowSettings;//
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

//TODO: add a way to change the window size
//Generate docs with cargo doc --open --viz
pub struct Game {
    gl: GlGraphics,//The opengl graphics
    rows: u32,//The number of rows
    cols: u32,//The number of columns
    snake: Snake,//The snake
    just_eaten: bool,//The snake just ate
    square_width: u32,//The width of a square
    food: Food,//The food
    score: u32,//The score
}
//self - method takes ownership
//&self - immutable borrow
//&mut self - mutable borrow
impl Game {
    fn render(&mut self, args: &RenderArgs) {//Render the game
        //use graphics;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];//The color of the snake

        self.gl.draw(args.viewport(), |_c, gl| {//Draw the snake
            graphics::clear(GREEN, gl);//Clear the screen
        });

        self.snake.render(args);//Draw the snake
        self.food.render(&mut self.gl, args, self.square_width);//Draw the food
    }

    fn update(&mut self, _args: &UpdateArgs) -> bool {//Update the game
        if !self.snake.update(self.just_eaten, self.cols, self.rows) {//Update the snake
            return false;//If the snake is dead
        }

        if self.just_eaten {//If the snake just ate
            self.score += 1;//Add 1 to the score
            self.just_eaten = false;//Reset the just_eaten flag
        }

        self.just_eaten = self.food.update(&self.snake);//Update the food
        if self.just_eaten {//If the snake just ate
            use rand::Rng;//Use the random number generator
            use rand::thread_rng;//Use the thread random number generator
            let mut r = thread_rng();//Get the random number generator
            loop {//Loop until we get a valid position
                let new_x = r.gen_range(0, self.cols);//Get a random x position
                let new_y = r.gen_range(0, self.rows);//Get a random y position
                if !self.snake.is_collide(new_x, new_y) {//If the position is valid
                    self.food = Food { x: new_x, y: new_y };//Set the food
                    break;//Break the loop
                }
            }
        }

        true//Return true
    }

    fn pressed(&mut self, btn: &Button) {//Handle the button press
        let last_direction = self.snake.d.clone();//Clone the last direction
        self.snake.d = match btn {//Handle the button press
            &Button::Keyboard(Key::Up) if last_direction != Direction::DOWN => Direction::UP,//Up
            &Button::Keyboard(Key::Down) if last_direction != Direction::UP => Direction::DOWN,//Down
            &Button::Keyboard(Key::Left) if last_direction != Direction::RIGHT => Direction::LEFT,//Left
            &Button::Keyboard(Key::Right) if last_direction != Direction::LEFT => Direction::RIGHT,//Right
            _ => last_direction,//Do nothing
        };
    }
}

#[derive(Clone, PartialEq)]//Derive the Clone trait and the PartialEq trait
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct Snake {//The snake
    gl: GlGraphics,//The opengl graphics
    snake_parts: LinkedList<SnakePiece>,//The snake parts
    width: u32,//The width of the snake
    d: Direction,//The direction of the snake
}

#[derive(Clone)]//Derive the Clone trait
pub struct SnakePiece(u32, u32);//The snake piece

impl Snake {//The snake
    pub fn render(&mut self, args: &RenderArgs) {//Render the snake

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];//The color of the snake

        let squares: Vec<graphics::types::Rectangle> = self.snake_parts//Get the snake parts
            .iter()//Get an iterator
            .map(|p| SnakePiece(p.0 * self.width, p.1 * self.width))//Map the snake parts to squares
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))//Map the squares to rectangles
            .collect();//Collect the squares

        self.gl.draw(args.viewport(), |c, gl| {//Draw the snake
            let transform = c.transform;//Get the transform

            squares//Draw the squares
                .into_iter()//Into the iterator
                .for_each(|square| graphics::rectangle(RED, square, transform, gl));//Draw the rectangles
        })
    }

    /// Move the snake if valid, otherwise returns false.
    pub fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {//Update the snake
        let mut new_front: SnakePiece =//Get the new front
            (*self.snake_parts.front().expect("No front of snake found.")).clone();//Clone the front

        if (self.d == Direction::UP && new_front.1 == 0)//If the direction is up and the new front is at the top
            || (self.d == Direction::LEFT && new_front.0 == 0)//If the direction is left and the new front is at the left
            || (self.d == Direction::DOWN && new_front.1 == rows - 1)//If the direction is down and the new front is at the bottom
            || (self.d == Direction::RIGHT && new_front.0 == cols - 1)//If the direction is right and the new front is at the right
        {
            return false;//The snake is dead
        }

        match self.d {
            Direction::UP => new_front.1 -= 1,//Move the snake up
            Direction::DOWN => new_front.1 += 1,//Move the snake down
            Direction::LEFT => new_front.0 -= 1,//Move the snake left
            Direction::RIGHT => new_front.0 += 1,//Move the snake right
        }

        if !just_eaten {//If the snake just ate
            self.snake_parts.pop_back();//Remove the tail
        }

        if self.is_collide(new_front.0, new_front.1) {//If the snake is colliding
            return false;//The snake is dead
        }

        self.snake_parts.push_front(new_front);//Add the new front
        true//Return true
    }

    fn is_collide(&self, x: u32, y: u32) -> bool {//Check if the snake is colliding
        self.snake_parts.iter().any(|p| x == p.0 && y == p.1)//Check if any part of the snake is at the position
    }
}

pub struct Food {//The food
    x: u32,//The x position
    y: u32,//The y position
}

impl Food {//The food
    // Return true if snake ate food this update
    fn update(&mut self, s: &Snake) -> bool {//Update the food
        let front = s.snake_parts.front().unwrap();//Get the front of the snake
        if front.0 == self.x && front.1 == self.y {//If the front of the snake is the food
            true
        } else {//Else
            false
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {//Render the food

        const BLACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];//The color of the food

        let x = self.x * width;//Get the x position
        let y = self.y * width;//Get the y position

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);//Get the square

        gl.draw(args.viewport(), |c, gl| {//Draw the food
            let transform = c.transform;//Get the transform

            graphics::rectangle(BLACK, square, transform, gl)//Draw the square
        });
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if this fails.
    let opengl = OpenGL::V3_2;//The opengl version

    const COLS: u32 = 30;//The number of columns
    const ROWS: u32 = 20;//The number of rows
    const SQUARE_WIDTH: u32 = 20;//The width of the squares

    let width = COLS * SQUARE_WIDTH;//The width of the screen
    let height = ROWS * SQUARE_WIDTH;//The height of the screen

    let mut window: GlutinWindow = WindowSettings::new("Snake Game", [width, height])//Create the window
        .opengl(opengl)//Set the opengl version
        .exit_on_esc(true)//Exit on pressing ESC
        .build()//Build the window
        .unwrap();//Get the window

    let mut game = Game {//Create the game
        gl: GlGraphics::new(opengl),//Create the graphics
        rows: ROWS,//The number of rows
        cols: COLS,//The number of columns
        square_width: SQUARE_WIDTH,//The width of the squares
        just_eaten: false,//The snake just ate
        food: Food { x: 1, y: 1 },//The food
        score: 0,//The score
        snake: Snake {//The snake
            gl: GlGraphics::new(opengl),//Create the graphics
            snake_parts: LinkedList::from_iter((vec![SnakePiece(COLS / 2, ROWS / 2)]).into_iter()),//The snake parts
            width: SQUARE_WIDTH,//The width of the squares
            d: Direction::DOWN,//The direction
        },
    };

    let mut events = Events::new(EventSettings::new()).ups(10);//Create the events
    while let Some(e) = events.next(&mut window) {//While there are events
        if let Some(r) = e.render_args() {//If there is a render args
            game.render(&r);//Render the game
        }

        if let Some(u) = e.update_args() {//If there is an update args
            if !game.update(&u) {//If the snake is dead
                break;
            }
        }

        if let Some(k) = e.button_args() {//If there is a button args
            if k.state == ButtonState::Press {//If the button is pressed
                game.pressed(&k.button);//Pressed the button
            }
        }
    }
    println!("Congratulations, your score was: {}", game.score);//Print the score
}