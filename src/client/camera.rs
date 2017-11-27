use vecmath;

pub struct Camera {
    x: f64,
    y: f64,
    zoom: f64,
    angle: f64,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    near: f32,
    far: f32
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            angle: 0.0,
            left: -500.0,
            right: 500.0,
            top: 500.0,
            bottom: -500.0,
            near: 10.0,
            far: 100.0
        }
    }

    pub fn update(&mut self, cursor_position: (f64, f64), viewport: &[f64; 2], dt: f64) {
        const EDGE_WIDTH: f64 = 30.0;
        const MAX_SPEED: f64 = 1.5;

        let speed_per_px = MAX_SPEED / EDGE_WIDTH;

        let window_width = viewport[0];
        let window_height = viewport[1];

        let (cursor_position_x, cursor_position_y) = cursor_position;

        if cursor_position_x >= (window_width - EDGE_WIDTH) {
            let mut speed = MAX_SPEED - (window_width - cursor_position_x) * speed_per_px;
            speed *= dt / self.zoom;

            self.x -= speed * self.angle.cos();
            self.y += speed * self.angle.sin();
        }

        if cursor_position_x <= EDGE_WIDTH {
            let mut speed = MAX_SPEED - cursor_position_x * speed_per_px;
            speed *= dt / self.zoom;

            self.x += speed * self.angle.cos();
            self.y -= speed * self.angle.sin();
        }

        if cursor_position_y >= (window_height - EDGE_WIDTH) {
            let mut speed = MAX_SPEED - (window_height - cursor_position_y) * speed_per_px;
            speed *= dt / self.zoom;

            self.y += speed * self.angle.cos();
            self.x += speed * self.angle.sin();
        }

        if cursor_position_y <= EDGE_WIDTH {
            let mut speed = MAX_SPEED - cursor_position_y * speed_per_px;
            speed *= dt / self.zoom;

            self.y -= speed * self.angle.cos();
            self.x -= speed * self.angle.sin();
        }
    }

    pub fn zoom_in(&mut self) {
        let zoom = self.zoom;
        self.set_zoom(zoom + 0.1);
    }

    pub fn zoom_out(&mut self) {
        let zoom = self.zoom;
        self.set_zoom(zoom - 0.1);
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.min(2.0).max(0.1);

        self.left = -500.0 * self.zoom as f32;
        self.right = 500.0 * self.zoom as f32;
        self.top = 500.0 * self.zoom as f32;
        self.bottom = -500.0 * self.zoom as f32;
    }

    pub fn view_matrix(&self, viewport: &[f64; 2]) -> [[f32; 4]; 4] {
        let width = viewport[0];
        let height = viewport[1];

        let aspect_ratio = height as f32 / width as f32;

        let lr = aspect_ratio * 2.0 / (self.right - self.left);
        let bt = 2.0 / (self.top - self.bottom);
        let nf = -2.0 / (self.far - self.near);

        let tx = - (self.right + self.left) / (self.right - self.left);
        let ty = - (self.top + self.bottom) / (self.top - self.bottom);
        let tz = - (self.far + self.near) / (self.far - self.near);

        let x = self.x as f32;
        let y = self.y as f32;

        [
            [ lr, 0.0, 0.0,  tx],
            [0.0,  bt, 0.0,  ty],
            [0.0, 0.0,  nf,  tz],
            [  x,   y, 0.0, 1.0],
        ]
    }

    pub fn unproject(&self, window_coordinates: (f64, f64), viewport: &[f64; 2]) -> (f32, f32) {
        let mut ndc = self.get_ndc(window_coordinates, viewport);

        ndc[0] -= self.x as f32;
        ndc[1] -= self.y as f32;

        let inv = vecmath::mat4_inv(self.view_matrix(viewport));
        let result = vecmath::row_mat4_transform(inv, ndc);

        (result[0], result[1])
    }

    fn get_ndc(&self, window_coordinates: (f64, f64), viewport: &[f64; 2]) -> [f32; 4] {
        let (x, y) = window_coordinates;

        let width = viewport[0];
        let height = viewport[1];

        [
            (x as f32) * 2.0 / (width as f32) - 1.0,
            -(y as f32) * 2.0 / (height as f32) + 1.0,
            -1.0,
            1.0
        ]
    }
}
