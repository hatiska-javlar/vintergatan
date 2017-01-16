use planet::PlanetClient;

pub enum ClientCommand {
    Process {
        planets: Vec<PlanetClient>
    }
}
