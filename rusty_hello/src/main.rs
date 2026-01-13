fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut name = "World".to_string();
    let mut upper = false;
    let mut repeat = 1;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--upper" => upper = true,
            "--repeat" => {
                repeat = args[i + 1].parse().unwrap_or(1);
                i += 1;
            }
            _ => name = args[i].clone(),
        }
        i += 1;
    }

    if upper {
        name = name.to_uppercase();
    }

    for _ in 0..repeat {
        let greeting = if upper {
            format!("HELLO, {}!", name)
        } else {
            format!("Hello, {}!", name)
        };
        println!("{}", greeting);
    }
}
