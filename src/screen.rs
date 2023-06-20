use minifb::Window;
use minifb::WindowOptions;
pub struct Screen{
    window: Window
}
impl Screen{
    pub fn new() -> Self{
        Self { 
            window: 
                match Window::new("Test", 640, 400, WindowOptions::default()) 
                    {
                        Ok(win) => win,
                        Err(err) => {
                            panic!("Unable to create window {}", err);
                        }
                    } 
            }
    }
    // pub fn window_show(&mut self){
    //     self.window.g
    // }
}
