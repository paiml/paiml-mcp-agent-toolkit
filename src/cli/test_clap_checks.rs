#[test]
fn test_clap_checks() {
    use crate::cli::Cli;
    use clap::Parser;
    let c = Cli::try_parse_from(["pmat", "comply", "check", "--checks", "CB-030,"]).unwrap();
    println!("comma: {:?}", c.command);
    let c = Cli::try_parse_from(["pmat", "comply", "check", "--checks", "CB-030", "--checks", "CB-030"]).unwrap();
    println!("dupe: {:?}", c.command);
    let c = Cli::try_parse_from(["pmat", "comply", "check", "--checks", " CB-030"]).unwrap();
    println!("space: {:?}", c.command);
}
