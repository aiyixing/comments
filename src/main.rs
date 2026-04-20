mod api;
mod crypto;

use clap::{Parser, Subcommand};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "wechat-comments")]
#[command(about = "微信公众号留言管理CLI工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "配置微信公众号凭证 (APPID和APPSECRET)")]
    Config {
        #[arg(short, long, help = "微信公众号APPID")]
        app_id: Option<String>,
        
        #[arg(short, long, help = "微信公众号APPSECRET")]
        app_secret: Option<String>,
    },

    #[command(about = "清除已保存的配置")]
    Clear,

    #[command(about = "查看配置状态")]
    Status,

    #[command(about = "打开已群发文章评论")]
    Open {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
    },

    #[command(about = "关闭已群发文章评论")]
    Close {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
    },

    #[command(about = "查看指定文章的评论数据")]
    List {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
        
        #[arg(short, long, default_value = "0", help = "开始位置")]
        begin: u32,
        
        #[arg(short, long, default_value = "10", help = "获取数量")]
        count: u32,
        
        #[arg(short, long, default_value = "0", help = "评论类型: 0精选评论, 1全部评论")]
        comment_type: u32,
    },

    #[command(about = "评论标记精选")]
    MarkElect {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
        
        #[arg(short, long, help = "评论ID")]
        comment_id: u64,
    },

    #[command(about = "评论取消精选")]
    UnmarkElect {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
        
        #[arg(short, long, help = "评论ID")]
        comment_id: u64,
    },

    #[command(about = "删除评论")]
    Delete {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
        
        #[arg(short, long, help = "评论ID")]
        comment_id: u64,
    },

    #[command(about = "回复评论")]
    Reply {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
        
        #[arg(short, long, help = "评论ID")]
        comment_id: u64,
        
        #[arg(short, long, help = "回复内容")]
        content: String,
    },

    #[command(about = "删除回复")]
    DeleteReply {
        #[arg(short, long, help = "消息数据ID")]
        msg_data_id: u64,
        
        #[arg(short, long, default_value = "0", help = "多图文时，指定第几篇文章")]
        index: u32,
        
        #[arg(short, long, help = "评论ID")]
        comment_id: u64,
        
        #[arg(short, long, help = "回复ID")]
        reply_id: u64,
    },
}

fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

fn print_json(result: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(result).unwrap_or_default());
}

fn exit_with_error(msg: &str) -> ! {
    let error = serde_json::json!({
        "error": msg
    });
    println!("{}", serde_json::to_string_pretty(&error).unwrap_or_default());
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Config { app_id, app_secret } => {
            let crypto_manager = crypto::CryptoManager::new();
            
            let id = app_id.clone().unwrap_or_else(|| prompt_input("请输入微信公众号APPID: "));
            let secret = app_secret.clone().unwrap_or_else(|| prompt_input("请输入微信公众号APPSECRET: "));
            
            if id.is_empty() || secret.is_empty() {
                exit_with_error("APPID和APPSECRET不能为空");
            }

            match crypto_manager.save_credentials(&id, &secret) {
                Ok(_) => {
                    let result = serde_json::json!({
                        "success": true,
                        "message": "配置已保存并加密",
                        "config_dir": crypto_manager.get_config_dir().to_string_lossy().to_string()
                    });
                    print_json(&result);
                }
                Err(e) => exit_with_error(&e),
            }
        }

        Commands::Clear => {
            let crypto_manager = crypto::CryptoManager::new();
            match crypto_manager.delete_credentials() {
                Ok(_) => {
                    let result = serde_json::json!({
                        "success": true,
                        "message": "配置已清除"
                    });
                    print_json(&result);
                }
                Err(e) => exit_with_error(&e),
            }
        }

        Commands::Status => {
            let crypto_manager = crypto::CryptoManager::new();
            let exists = crypto_manager.credentials_exist();
            let config_dir = crypto_manager.get_config_dir().to_string_lossy().to_string();
            
            let result = serde_json::json!({
                "config_exists": exists,
                "config_dir": config_dir,
                "message": if exists { "配置文件存在" } else { "配置文件不存在，请使用 config 命令进行配置" }
            });
            print_json(&result);
        }

        _ => {
            let mut client = match api::WechatApiClient::new().await {
                Ok(c) => c,
                Err(e) => exit_with_error(&e),
            };

            let result: Result<serde_json::Value, String> = match &cli.command {
                Commands::Open { msg_data_id, index } => {
                    client.open_comment(*msg_data_id, *index).await
                }
                Commands::Close { msg_data_id, index } => {
                    client.close_comment(*msg_data_id, *index).await
                }
                Commands::List { msg_data_id, index, begin, count, comment_type } => {
                    client.list_comments(*msg_data_id, *index, *begin, *count, *comment_type).await
                }
                Commands::MarkElect { msg_data_id, index, comment_id } => {
                    client.mark_elect_comment(*msg_data_id, *index, *comment_id).await
                }
                Commands::UnmarkElect { msg_data_id, index, comment_id } => {
                    client.unmark_elect_comment(*msg_data_id, *index, *comment_id).await
                }
                Commands::Delete { msg_data_id, index, comment_id } => {
                    client.delete_comment(*msg_data_id, *index, *comment_id).await
                }
                Commands::Reply { msg_data_id, index, comment_id, content } => {
                    client.reply_comment(*msg_data_id, *index, *comment_id, content).await
                }
                Commands::DeleteReply { msg_data_id, index, comment_id, reply_id } => {
                    client.delete_reply(*msg_data_id, *index, *comment_id, *reply_id).await
                }
                _ => unreachable!(),
            };

            match result {
                Ok(r) => print_json(&r),
                Err(e) => exit_with_error(&e),
            }
        }
    }
}
