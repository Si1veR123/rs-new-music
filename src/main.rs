mod artists;
mod songs;
use artists::{new_artist, remove_artist};
use songs::recent_songs;

use colored;

fn main() {
    let _ = colored::control::set_virtual_terminal(true);

    let mut args = std::env::args();
    let _  = args.next();
    let function = args.next().unwrap_or("".to_string());

    let remaining_args = args.collect::<Vec<String>>().join(" ");

    match function.as_str() {
        "new" => new_artist(&remaining_args),
        "remove" => remove_artist(&remaining_args),
        d => {
            recent_songs(d.parse::<u64>().unwrap_or(7));
        },
    }

    
}
