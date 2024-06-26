// triggers.rs
pub fn trigger_words() -> Vec<&'static str> {
    vec![
        "generate", "create", "make", "produce", "design", "draw", "build", "elaborate",
        "painting", "paint", "prepare", "formulate", "render", "illustrate", "sketch",
        "show me", "i want to see", "craft", "picture of", "image of", "cityscape",
        "landscape", "bored ape", "cinematic", "bokeh", "color grading", "cat", "dog",
        "home", "mountain", "cinema", "photo", "cartoon", "visualize", "depict",
        "portray", "represent", "envisage", "conceive", "fabricate", "manufacture",
        "compose", "assemble", "art", "graphic", "illustration", "picture", "image",
        "photo", "photograph", "snapshot", "scene", "setting", "background", "foreground",
        "realistic", "stylized", "abstract", "surreal", "fantasy", "sci-fi", "retro",
        "vintage", "modern", "minimalist", "detailed", "intricate", "space", "galaxy",
        "universe", "nature", "wildlife", "city", "urban", "rural", "fantasy creatures",
        "mythical", "historical", "futuristic", "architecture", "interior design", "dark",
        "light", "moody", "bright", "colorful", "monochrome", "gritty", "soft", "dreamy",
        "nighttime", "daytime", "car", "vehicle", "animal", "plant", "tree", "flower",
        "building", "structure", "machine", "robot", "food", "drink", "in the style of",
        "inspired by", "based on", "similar to", "like a", "in the manner of", "with a",
        "featuring a", "including a", "superman", "pepe", "brainstorm", "ape", "forest",
        "samurai", "savage", "ninja", "human", "buttock", "humanoid", "body", "detective",
        "croissant", "paris", "skateboarding", "boulevard", "afternoon", "splendid",
        "20th century", "forge", "devise", "digitize", "morph", "filter", "dark theme",
        "light tones", "comic style", "anime", "manga", "expressionism", "impressionism",
        "cubism", "pop art", "ocean", "desert", "jungle", "superhero", "zombie", "pirate",
        "cyberpunk", "steampunk", "post-apocalyptic", "drone", "exoskeleton", "sports car",
        "sailing ship", "mammal", "reptile", "cactus", "orchid", "exploding", "melting",
        "reflecting", "sunset", "battlefield", "concert", "medieval", "renaissance",
        "industrial age", "Victorian", "Nordic", "Mayan", "sunrise", "beach", "add a",
        "change the", "make another", "one more", "replace the", "try another"
    ]
}

pub fn contains_trigger_word(input: &str) -> bool {
    let triggers = trigger_words();
    triggers.iter().any(|&trigger| input.to_lowercase().contains(trigger))
}
