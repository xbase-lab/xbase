use super::NvimGlobal;
use crate::runtime::{rpc, rt};
use crate::BroadcastHandler;
use mlua::{chunk, prelude::*};
use once_cell::sync::Lazy;
use os_pipe::{PipeReader, PipeWriter};
use std::os::unix::io::IntoRawFd;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::{collections::HashMap, io::Write, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};
use xbase_proto::*;

static BROADCASTERS: Lazy<Mutex<HashMap<PathBuf, JoinHandle<Result<()>>>>> =
    Lazy::new(Default::default);

pub struct Broadcast;

impl Broadcast {
    /// Register a project and initialize command listener if the project isn't already initialized
    pub fn init_or_skip(lua: &Lua, root: &PathBuf) -> LuaResult<()> {
        let mut broadcast = BROADCASTERS.lock().unwrap();
        if !broadcast.contains_key(root) {
            let (reader, writer) = os_pipe::pipe()?;

            Broadcast::start_reader(lua, reader)?;
            let writer = Broadcast::start_writer(writer, root.clone());
            broadcast.insert(root.clone(), writer);
        }
        Ok(())
    }

    /// Main handler of daemon messages
    fn handle(lua: &Lua, line: LuaString) -> LuaResult<()> {
        match lua.parse(line.to_string_lossy().into()) {
            Ok(msgs) => {
                for msg in msgs {
                    lua.handle(msg)?;
                }
                Ok(())
            }
            Err(err) => {
                lua.error(err.to_string()).ok();
                Ok(())
            }
        }
    }

    /// Setup and load a uv pipe to call [`Self::handle`] with read bytes
    pub fn start_reader(lua: &Lua, reader: PipeReader) -> LuaResult<()> {
        let reader_fd = reader.into_raw_fd();
        let callback = lua.create_function(Self::handle)?;

        // TODO: should closing be handled?
        lua.load(chunk! {
            local pipe = vim.loop.new_pipe()
            pipe:open($reader_fd)
            pipe:read_start(function(err, chunk)
                assert(not err, err)
                if chunk then
                    vim.schedule(function()
                         $callback(chunk)
                     end)
                end
            end)
        })
        .exec()
    }

    pub fn start_writer(mut writer: PipeWriter, root: PathBuf) -> JoinHandle<Result<()>> {
        std::thread::spawn(move || {
            rt().block_on(async move {
                let rpc = rpc().await;
                let address = rpc.register(context::current(), root).await??;
                let mut stream = UnixStream::connect(address).await?;
                drop(rpc);

                let (reader, _) = stream.split();
                let mut breader = BufReader::new(reader);
                let mut line = vec![];

                while let Ok(len) = breader.read_until(b'\n', &mut line).await {
                    if len == 0 {
                        break;
                    }

                    writer.write_all(line.as_slice()).ok();

                    line.clear();
                }

                OK(())
            })?;

            OK(())
        })
    }
}
