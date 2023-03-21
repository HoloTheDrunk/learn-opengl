use std::str::SplitWhitespace;

pub type Coords = [f32; 3];

#[derive(Clone, Debug, Default)]
struct Object {
    name: String,
    vertices: Vec<Coords>,
    normals: Vec<Coords>,
    faces: Vec<(usize, Option<usize>, Option<usize>)>,
}

impl Object {
    pub fn load(path: &str) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let mut object = Self::default();

        for (line, line_content) in content.lines().enumerate() {
            if line_content.is_empty() || line_content.chars().next().unwrap_or('#') == '#' {
                continue;
            }

            let mut tokens = line_content.split_whitespace();
            let marker = tokens.next().unwrap();

            match marker {
                "o" => {
                    let name = tokens.next().unwrap();
                    println!("Parsing object `{name}`");

                    object = Object {
                        name: name.to_owned(),
                        ..Default::default()
                    };
                }
                "g" => println!("Parsing group `{}`", tokens.next().unwrap()),
                "s" => println!(
                    "Smooth shading would now be {}",
                    match tokens.next().unwrap() {
                        "1" | "on" => "on",
                        "0" | "off" => "off",
                        v => panic!("Unhandled smooth shading setting `{v}`"),
                    }
                ),
                "v" => object.push_coords(line, tokens),
                "vn" => object.push_coords(line, tokens),
                "f" => object.push_face(line, tokens),
                _ => panic!("Unhandled marker {marker}"),
            }
        }

        Ok(object)
    }

    pub fn vertices(&self) -> Vec<Coords> {
        self.faces
            .iter()
            .map(|(index, _, _)| self.vertices[*index])
            .collect()
    }

    pub fn normals(&self) -> Vec<Coords> {
        self.faces
            .iter()
            .map(|(_, _, opt_index)| {
                opt_index
                    .map(|index| self.normals[index])
                    .unwrap_or([0., 0., 0.])
            })
            .collect()
    }

    fn push_coords(&mut self, line: usize, tokens: SplitWhitespace) {
        let coords = tokens
            .map(|token| {
                token
                    .parse::<f32>()
                    .expect(format!("Failed to parse coords, should be an f32: {token}").as_str())
            })
            .collect::<Vec<_>>();

        if !(3..4).contains(&coords.len()) {
            panic!("Invalid coordinate count at line {line}");
        }

        self.vertices.push([coords[0], coords[1], coords[2]]);
    }

    fn push_face(&mut self, line: usize, tokens: SplitWhitespace) {
        let indices = tokens
            // Keeping only the vertex indices
            .map(|token| {
                let indices = parse_indices(token);
                (
                    indices[0].unwrap() - 1,
                    indices
                        .get(1)
                        .unwrap_or(&None)
                        .to_owned()
                        .map(|index| index - 1),
                    indices
                        .get(2)
                        .unwrap_or(&None)
                        .to_owned()
                        .map(|index| index - 1),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            3,
            indices.len(),
            "Invalid vertex count for face at line {line} (should be 3, is {})",
            indices.len()
        );

        self.faces.extend(indices.iter());
    }
}

fn parse_indices(string: &str) -> Vec<Option<usize>> {
    string
        .split('/')
        .map(|index| index.parse::<usize>().ok())
        .collect()
}
