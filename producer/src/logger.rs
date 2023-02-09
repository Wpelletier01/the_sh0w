#![allow(dead_code)]
#![allow(non_snake_case)]




use std::sync::{Mutex, MutexGuard};
use std::time::{Duration,Instant};

use colored::{Colorize,ColoredString};
use error_stack::{Result, ResultExt};

use super::EGeneral;

#[cfg(debug_assertions)]
pub(crate) const RELEASE:u8 = 0;

#[cfg(not(debug_assertions))]
pub(crate) const RELEASE:u8 = 1;

pub const FHOUR_AS_SECONDS: f32 =   3600.0;
pub const FMIN_AS_SECONDS:  f32 =   60.0;

// ------------------------------------------------------------------------------------------------
// Clock information
//
//
lazy_static::lazy_static! {

    static ref APPS_CLOCK:  Mutex<Instant> = Mutex::new(Instant::now());

}


/// Return the elapsed time since the program started
pub fn get_prog_elapsed_time() -> Duration {
    //
    match APPS_CLOCK.lock() {

        Ok(clock) => clock.elapsed(),
        Err(e) => {

            eprintln!("unable to access the engine's clock because: {}", e.to_string());
            eprintln!("a duration value of 0 will be returned");

            Duration::new(0, 0)

        }

    }
    //
}
//
//
//
// TODO: add function to write the logs to a file
//
//
// ------------------------------------------------------------------------------------------------
// Test
//
#[cfg(test)]
mod test {

    use super::{init,CDEBUGS};

    #[test]
    fn logs_with_argument() {

        init();

        CDEBUGS("Test with one value {}" ,&[&1.to_string()]);
        CDEBUGS("Test with two value {} {}",&[&1.to_string(),&2.to_string()]);
        CDEBUGS("Test with three value {} {} {}",&[&1.to_string(),&2.to_string(),&3.to_string()]);

    }

    #[test]
    fn log_with_to_much_arg() {

        init();

        CDEBUGS("1 {} 2 {} and 3",&["test","test","One more"]);

    }


}
//
//
// ------------------------------------------------------------------------------------------------
// shouldn't occur a lot in this engine but this global variable is needed
//
lazy_static::lazy_static! {
    //
    static ref LOG_SYSTEM: Mutex<LogSystem> = Mutex::new(
        LogSystem {
            queue:      LogQueue::new(),
            init:       false,
            debug_log:  true,
            info_log:   true,
            warn_log:   true,
            trace_log:  true,
            vulkan:     true
        }
    );
    //
}
//
// ------------------------------------------------------------------------------------------------
// Constant
//
const       LOG_BUFFER_SIZE:    usize       = 20000;
const       LOG_MAX_QUEUE_SIZE: usize       = 300;
const       MAX_LINE_LEN:       usize       = 100;
const       CFAILURE:           u8          = 0;
const       CSUCCESS:           u8          = 1;
const       LEVEL_STRING:       [&str;7]    = [
    "[FATAL]:","[ERROR]:","[WARN]: ","[INFO]: ", "[DEBUG]:","[TRACE]:","[VLK]:  "
];
// tab jump for if a log have multiple lines, they start all at the same position
const TAB_MESSAGE: &str = "\n                       "; // 23 columns of whitespace
//
//
// ------------------------------------------------------------------------------------------------
// The log subsystem
//
//
/// creates and sends a log messages through out the Engine
struct LogSystem {

    queue:      LogQueue,
    init:       bool,
    debug_log:  bool,
    info_log:   bool,
    warn_log:   bool,
    trace_log:  bool,
    vulkan:     bool

}
//
impl LogSystem {
    //
    /// Initialize the log subsystem
    fn initialize(&mut self) {
        //
        // they are all enable by default
        //
        // check if in release mode and if so disable debug and trace logging
        if RELEASE == 1{

            self.debug_log = false;
            self.info_log  = false;

        }
        //
        self.init = true;
        // add the initialize log
        //CTRACE("Start the log subsystem");

        //
        //
    }
    //
    //
    /// Add a log entry to the end of the LOG_QUEUE
    ///
    /// # Parameters
    ///
    /// * log - a log entry to be added to the queue
    ///
    fn push_log(&mut self,level: Level,msg: &str) {
        //
        // check to make sure that the log subsystem is initialized
        if !self.is_init(){
            // TODO: found a better solution because the log subsystem
            //       should not make the application crash
            panic!("try to add a log but the log subsystem is not initialized");
            //
        }
        //
        // check if the level is enabled and then pass it to the queue
        match level {
            //
            Level::DEBUG => {

                if self.debug_log {

                    self.queue.push(Log::new(level, msg.green()));

                }

            },
            //
            Level::INFO => {

                if self.info_log {

                    self.queue.push(Log::new(level, msg.blue()));

                }

            },
            //
            Level::TRACE => {

                if self.trace_log {

                    self.queue.push(Log::new(level, msg.magenta()));

                }

            },
            //
            Level::WARN => {

                if self.warn_log {

                    self.queue.push(Log::new(level,msg.yellow()));

                }

            },
            //
            // Fatal and Error types are not allowed to be disabled so no need to be checked
            _ => self.queue.push(Log::new(level,msg.red()))
            //
            //
        }
        //
        self.print();
        //
    }
    //
    /// wrapper for the macro println! but only for the
    /// log message this function will probably be unable to be used by default
    fn print(&self) {
        //
        println!("{}", match self.queue.content.last(){

            Some(v) => v.as_string(),
            None => "".to_string(),

        });
        //
    }
    //
    /// Check if the sub system logging have been initialize
    fn is_init(&self) -> bool { self.init }
    //
    //
    /// Change the status of the Info log type
    pub(crate) fn set_info(&mut self,value:bool) { self.info_log = value; }
    //
    /// Change the status of the Trace log type
    pub(crate) fn set_trace(&mut self,value:bool) { self.trace_log = value; }
    //
    //
}
//
impl Drop for LogSystem {

    fn drop(&mut self) {

        let mutx = get_access_mutex().change_context(EGeneral::LogSys)
            .attach_printable("Cant drop the log system mutex")
            .unwrap();

        drop(mutx)

    }

}
//
//
/// Initialize the log system
pub fn init() -> Result<(),EGeneral> {
    //

    get_access_mutex().change_context(EGeneral::LogSys)
        .attach_printable("Can't initialize the Log System")?
        .initialize();

    Ok(())

}
//
//
/// remove boilerplate code for accessing the log system
fn get_access_mutex() -> Result<MutexGuard<'static,LogSystem>,EGeneral> {

    match LOG_SYSTEM.lock() {

        Ok(sys) => Ok(sys),
        Err(e) => return Err(
            EGeneral::LogSys
                .as_report()
                .attach_printable(
                    format!(
                        "Can't access LOG_SYSTEM caused by {}",
                        e.to_string()
                    )
                )
        )
    }

}
//
//
// ------------------------------------------------------------------------------------------------
// Formatting functions
//
//
/// Parsing log entry to a string
///
/// # Parameters
///
/// * level - the type of log
/// * message - what the log says
///
fn fmt_log(level: Level, message: String) -> String{
    //
    // level represent the index in the CLEVEL_STRING
    // " {TIME} {TYPE} {MESSAGE}"
    let msg = format!(
        "{} {} {}",
        fmt_duration_log(get_prog_elapsed_time()),
        LEVEL_STRING[level as usize],
        message
    );

    msg
    //
}
//
//
/// Format the duration entry to a string
///
/// # Parameters
///
/// * dur - Duration since Engine initialized
///
fn fmt_duration_log(dur:Duration) -> String {

    let mut secs = dur.as_secs_f32();
    let mut min:u32 = 0;
    let mut hour:u32 = 0;

    if secs >= FHOUR_AS_SECONDS {

        hour = (secs / FHOUR_AS_SECONDS) as u32;
        secs -= FHOUR_AS_SECONDS*hour as f32;

    }

    if secs >= FMIN_AS_SECONDS {

        min = (secs / FMIN_AS_SECONDS) as u32;
        secs -= FMIN_AS_SECONDS*min as f32;

    }

    let millis_sec = ((secs - (secs as u32) as f32 ) * 100_f32) as u32;

    let sec = secs as u32;

    let f_hour =    format_single_digit_value(hour);
    let f_min =     format_single_digit_value(min);
    let f_secs =    format_single_digit_value(sec);
    let f_millis =  format_single_digit_value(millis_sec);

    format!("[{}:{}:{}:{}]",f_hour,f_min,f_secs,f_millis)

}
//
//
/// parse an unsigned integer to a string and if it is a single digit number, it add a zero
/// before it
///
/// # Arguments
///
/// * 'value' - a unsigned integer to parse into a string
///
fn format_single_digit_value(value: u32) -> String {
    //
    if value < 9 {
        return format!("0{}",value);
    }

    format!("{}",value)
    //
}
//
// ------------------------------------------------------------------------------------------------
// Log Struct
//
/// Vector that store every log entry
struct LogQueue { content: Vec<Log> }
//
impl LogQueue {
    //
    /// initialize the queue
    fn new() -> Self {
        //
        let q :Vec<Log> = Vec::with_capacity(LOG_MAX_QUEUE_SIZE);

        LogQueue{ content: q}
        //
    }
    //
    /// add a log entry to the end queue
    ///
    /// # Arguments
    ///
    /// * 'log' - a Log entry to be add
    ///
    fn push(&mut self,log:Log) {
        //
        if self.content.len() + 1 == LOG_MAX_QUEUE_SIZE {

            //TODO: implement something for remedy this situation
            //      I just dont have idea for now
            unimplemented!();

        }

        self.content.push(log);
        //
    }
    //
    //
}
//
//
/// Store a log entry
struct Log{ level: Level, content:String }
//
impl Log{
    //
    /// initialize a new log entry
    ///
    /// # Arguments
    ///
    /// * 'level'   - type of log entry
    /// * 'message' - colored message that the log entry should show
    ///
    fn new(level:Level, message:ColoredString) -> Self {
        //
        // format version message
        #[allow(unused_assignments)]
            let mut fmt_msg = String::new();
        //
        if message.len() > MAX_LINE_LEN {
            //
            // get the message color and if none set default one (white)
            let msg_color = match message.fgcolor() {

                Some(color) => color,
                None => colored::Color::White

            };
            //
            // how many lines the message should have
            let nb_lines = message.len() as f32 / MAX_LINE_LEN as f32;
            //
            // will store each line
            let mut lines: Vec<String> = Vec::new();
            //
            //
            for line in 0..nb_lines.ceil() as i32 {
                //
                // store the 100 range of characters depending on the value of 'line'
                lines.push(message[line as usize*MAX_LINE_LEN..].to_string());
                //
            }
            // align the multiple lines together
            fmt_msg = lines.join(TAB_MESSAGE).color(msg_color).to_string();
            //
            //
        } else{

            fmt_msg = message.to_string();

        }
        //
        //
        // add the header to the message
        let mut msg = fmt_log(level,fmt_msg);
        //
        // check if the len of the message is bigger than the max allowed
        if msg.len() > LOG_BUFFER_SIZE - 1 {

            msg = msg[..LOG_BUFFER_SIZE].to_string();

        }

        Log{ level: level, content: msg}
        //
        //
    }
    //
    /// return the log as a string
    pub fn as_string(&self) -> String { self.content.to_string() }
    //
    //
}
//
//
// ------------------------------------------------------------------------------------------------
// Level enum
//
#[repr(usize)]
#[derive(PartialEq,Clone, Copy)]
/// represents the index of the type of log in the const LEVEL_STRING
pub enum Level{

    FATAL = 0,
    ERROR = 1,
    WARN  = 2,
    INFO  = 3,
    DEBUG = 4,
    TRACE = 5,
    VLK   = 6,

}
//
//
// ------------------------------------------------------------------------------------------------
// Log call functions
//
//
//
/// Slice the given string where brackets couple are founded
///
/// # Arguments
///
/// * 'msg' - the message to be slice
///
fn slice_brackets_str(msg:&str) -> (Vec<String>,usize){

    let mut slices:Vec<String> = Vec::new();

    // iterator index through all the characters of the message
    let mut f_index:usize = 0;
    // iterator over how many {} founded
    let mut founding:usize = 0;

    for (index,c) in msg.chars().enumerate() {

        // if we found this '{}' we slice the message there
        if c == '{' && msg.chars().nth(index + 1) == Some('}') {

            let slice = &msg[f_index .. index];

            // add the left side of the {} founded
            slices.push(slice.to_string());
            // add {} to be able to replaced them with arguments
            slices.push("{}".to_string());

            // then we jump over {}
            f_index = index + 2;

            founding += 1;


        }
    }
    // get the last slice and add it with the others
    let slice_end = &msg[f_index .. msg.chars().count()];

    slices.push(slice_end.to_string());
    //
    (slices,founding)
    //
    //
}
//
//
/// validate that the string passed is a valid string with
/// valid number of arguments
///
/// # Arguments
///
/// * 'msg' - the string message to be validated
/// * 'args' - the arguments to be validated and passed to the message
fn validate_msg(msg: &str,args:&[&str]) -> String {
    //
    let (msg_sliced,nb_brackets) = slice_brackets_str(&msg);
    //
    // validate that the number of arguments passed are the same as the number of
    // couple brackets
    if args.len() > nb_brackets {

        panic!(
            "the message:[ {} ] contains {} format bracket(s) but you have passed {} arguments ",
            msg,
            nb_brackets,
            args.len()
        );

    }
    //
    // replace each bracket by the arguments
    let mut iter_brk:usize = 0;
    let mut f_msg = String::new();

    for slice in msg_sliced.iter() {

        if slice != &"{}" {

            f_msg = format!("{}{}",f_msg,slice);

        }
        else if args.len() > iter_brk as usize {

            f_msg = format!("{}{}",f_msg,args[iter_brk]);

            iter_brk += 1;
        }

    }
    //
    f_msg
    //
    //
}
//
//
/// Fatal log with no arguments
pub fn CFATAL(msg:&str) {

    match get_access_mutex() {

        Ok(mut sys) => sys.push_log(Level::FATAL, msg),

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Fatal log with arguments
pub fn CFATALS(msg:&str,args:&[&str]) {

    match get_access_mutex() {

        Ok(mut sys) => {

            let v = validate_msg(msg, args);

            sys.push_log(Level::FATAL,&v);

        },

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Error log with no arguments
pub fn CERROR(msg:&str) {

    match get_access_mutex() {

        Ok(mut sys) => sys.push_log(Level::ERROR, msg),

        Err(e) => eprintln!("{}",e.to_string())


    }
}
//
/// Error log with arguments
pub fn CERRORS(msg:&str,args:&[&str]) {

    match get_access_mutex() {

        Ok(mut sys) => {

            let v = validate_msg(msg, args);

            sys.push_log(Level::ERROR,&v);

        },

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Warn log with no arguments
pub fn CWARN(msg:&str) {

    match get_access_mutex() {

        Ok(mut sys) => sys.push_log(Level::WARN, msg),

        Err(e) => { eprintln!("{}",e.to_string()) }

    }
}
//
/// Warn log with arguments
pub fn CWARNS(msg:&str,args:&[&str]) {

    match get_access_mutex() {

        Ok(mut sys) => {

            let v = validate_msg(msg, args);

            sys.push_log(Level::WARN,&v);

        },

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Info log with no arguments
pub fn CINFO(msg:&str) {

    match get_access_mutex() {

        Ok(mut sys) => sys.push_log(Level::INFO, msg),

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Info log with arguments
pub fn CINFOS(msg:&str,args:&[&str]) {

    match get_access_mutex() {

        Ok(mut sys) => {

            let v = validate_msg(msg, args);

            sys.push_log(Level::INFO,&v);

        },

        Err(e) => eprintln!("{}",e.to_string())

    }
}
//
/// Debug log with no arguments
pub fn CDEBUG(msg:&str) {

    match get_access_mutex() {

        Ok(mut sys) => sys.push_log(Level::DEBUG, msg),

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Debug log with arguments
pub fn CDEBUGS(msg:&str,args:&[&str]) {

    match get_access_mutex() {

        Ok(mut sys) => {

            let v = validate_msg(msg, args);

            sys.push_log(Level::DEBUG,&v);

        },

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Trace log with no arguments
pub fn CTRACE(msg:&str) {

    match get_access_mutex() {

        Ok(mut sys) => sys.push_log(Level::TRACE, msg),

        Err(e) => eprintln!("{}",e.to_string())

    }

}
//
/// Trace log with arguments
pub fn CTRACES(msg:&str,args:&[&str]) {

    match get_access_mutex() {

        Ok(mut sys) => {

            let v = validate_msg(msg, args);

            sys.push_log(Level::TRACE,&v);

        },

        Err(e) => eprintln!("{}",e.to_string())

    }

}
