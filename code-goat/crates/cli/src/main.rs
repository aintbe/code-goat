use judger::{ResourceLimit, RunSpec};

fn main() {
    env_logger::init();

    let resource_limit = ResourceLimit::new(
        Some(12 * 1024 * 1025),
        Some(8 * 1000),
        Some(199 * 1000),
        None,
        None,
        None,
    );

    const BASE_DIR: &str = "/workspaces/code-goat/tests";
    const EXAMPLE_DIR: &str = "/long-loop";

    let work_path = format!("{}{}{}", BASE_DIR, EXAMPLE_DIR, "/ac/cpp");
    let test_path = format!("{}{}{}", BASE_DIR, EXAMPLE_DIR, "/testcases");

    let spec = RunSpec::new(
        format!("{}{}", work_path, "/main.o").as_ref(),
        None,
        Some(format!("{}{}", work_path, "/1.out").as_ref()),
        Some(format!("{}{}", work_path, "/1.error").as_ref()),
        None,
        [].to_vec(),
        [].to_vec(),
        resource_limit,
    );

    let result = judger::judge(spec);
    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap_or("".to_string())
    );
}
