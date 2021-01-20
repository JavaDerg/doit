use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "doit")]
pub struct CliConfig {
    /// Will drop into shell of target user
    /// ( -s is equal to doit su <user> ; -ss is equal to doit su - <user> (clean environment))
    #[structopt(name = "shell", short, parse(from_occurrences))]
    pub shell: u8,

    /// User<id> that will be logged into
    #[structopt(long, short = "i")]
    pub target_id: Option<u32>,

    /// User<name> that will be logged into
    #[structopt(long, short = "n")]
    pub target_name: Option<String>,

    #[structopt(name = "command")]
    pub command: Vec<String>,
}
