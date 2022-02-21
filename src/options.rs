pub fn extract_app_ids(args: &Vec<String>) -> Vec<u32> {
    let app_id_args = &args[1..];

    app_id_args
        .iter()
        .map(|arg| arg.trim().parse::<u32>().unwrap())
        .collect::<Vec<u32>>()
}
