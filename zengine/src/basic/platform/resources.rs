extern crate sdl2;

use crate::core::Resource;
use sdl2::video::GLContext;
use sdl2::Sdl;
use sdl2::VideoSubsystem;

impl Resource for VideoSubsystem {}

impl Resource for Sdl {}
