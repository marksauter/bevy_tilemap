//! ![](https://github.com/joshuajbouw/bevy_tilemap/raw/master/docs/img/logo.gif)
//!
//! # Bevy Tilemap
//!
//! Bevy Tilemap allows for Bevy native batch-rendered tiles in maps to be
//! constructed with chunk based loading, efficiently.
//!
//! Simple yet refined in its implementation, it is meant to attach to other
//! extensible plugins that can enhance its functionality further. Hand-crafted
//! tilemaps with an attentive focus on performance, and low data usage.
//!
//! ## Features
//! * Perfect for game jams.
//! * Easy to use and stable API with thorough documentation.
//! * Endless or constrained tilemaps.
//! * Batched rendering of many tiles.
//! * Square and hex tiles.
//!
//! ## Design
//! This is not intended to be just another Tilemap. It is meant to be a
//! framework and extensible by design, like Bevy. As well as work done to keep
//! it as close to Bevy API as possible while keeping in mind of Rust API best
//! practices. It is not meant to be complicated and created to be simple to use
//! but give enough functionality to advanced users.
//!
//! Less time fiddling, more time building.
//!
//! # Serde support
//!
//! Optionally serde is supported through the use of features.
//!
//! ```toml
//! [dependencies]
//! bevy_tilemap = { version = "0.4", features = ["serialize"] }
//! ```
//!
//! # Extra types feature
//!
//! Internally, the library uses Point2, Point3, Dimension2 and Dimension3
//! types. This is not part of the official Bevy library and multiple or
//! alternative implementations of them may not be ideal, especially not in the
//! prelude.
//!
//! It is quite important however to `impl Into<T>` for each of them for most
//! public methods. It already has most basic implementations that make sense.
//!
//! However if you would like to use this, please do so.
//!
//! ```toml
//! [dependencies]
//! bevy_tilemap = { version = "0.4", features = ["types"] }
//! ```
//!
//! See the library `bevy_tilemap_types` for more information.

#![doc(html_root_url = "https://docs.rs/bevy_tilemap/0.4.0")]
// This was broken even further and no longer will work at all with the previous
// workaround. There is a fix, might be sometime for it to be included though.
// Even then, it is just a warning. For now, including it per module seems to
// fix it.
// See: https://github.com/rust-lang/rust/pull/80372
// #![no_implicit_prelude]

// rustc / rustdoc
// This won't build on stable releases until it is stable.
//#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![deny(dead_code, unused_imports)]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(
    clippy::cast_lossless,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::panic_in_result_fn,
    clippy::print_stdout,
    clippy::unwrap_in_result
)]

#[cfg(feature = "types")]
pub extern crate bevy_tilemap_types;

#[cfg(feature = "types")]
#[doc(inline)]
pub use bevy_tilemap_types::dimension;
#[cfg(feature = "types")]
#[doc(inline)]
pub use bevy_tilemap_types::point;

#[no_implicit_prelude]
pub mod chunk;
#[no_implicit_prelude]
pub mod default_plugin;
#[no_implicit_prelude]
pub mod entity;
#[no_implicit_prelude]
pub mod prelude;
#[no_implicit_prelude]
pub mod stage {
    //! The stages for the tilemap in the bevy app.

    /// The tilemap stage, set to run before `POST_UPDATE` stage.
    pub const TILEMAP: &str = "tilemap";
    // pub const TILEMAP_UPDATE: &str = "tilemap_update";
}
#[no_implicit_prelude]
pub mod event;
#[no_implicit_prelude]
mod system;
#[no_implicit_prelude]
pub mod tile;
#[no_implicit_prelude]
pub mod tilemap;

use crate::{event::TilemapChunkEvent, lib::*};
pub use crate::{
    tile::Tile,
    tilemap::{Tilemap, TilemapLayer},
};

/// The Bevy Tilemap 2D main plugin.
#[derive(Default)]
pub struct TilemapPlugin;

/// The tilemap system stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum TilemapSystem {
    /// The events stage.
    Events,
    /// The auto spawn stage.
    AutoSpawn,
}

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<Tilemap>()
            .add_stage_before(
                CoreStage::PostUpdate,
                stage::TILEMAP,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                stage::TILEMAP,
                crate::system::tilemap_events
                    .system()
                    .label(TilemapSystem::Events),
            )
            .add_system_to_stage(
                stage::TILEMAP,
                crate::chunk::system::chunk_update
                    .system()
                    .after(TilemapSystem::Events),
            )
            .add_system_to_stage(
                stage::TILEMAP,
                crate::chunk::system::chunk_auto_radius
                    .system()
                    .after(TilemapSystem::Events),
            )
            .add_system_to_stage(
                stage::TILEMAP,
                crate::chunk::system::chunk_auto_spawn
                    .system()
                    .label(TilemapSystem::AutoSpawn)
                    .after(TilemapSystem::Events),
            )
            .add_system_to_stage(
                stage::TILEMAP,
                crate::system::tilemap_visibility_change.system(),
            );

        let world = app.world_mut().cell();
        // let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();
        let mut pipelines = world
            .get_resource_mut::<Assets<PipelineDescriptor>>()
            .unwrap();
        let mut shaders = world.get_resource_mut::<Assets<Shader>>().unwrap();
        crate::chunk::render::add_tilemap_graph(&mut pipelines, &mut shaders);
    }
}

/// A custom prelude around everything that we only need to use.
#[no_implicit_prelude]
mod lib {
    extern crate bevy;
    pub extern crate bitflags;
    #[cfg(feature = "serde")]
    extern crate serde;
    extern crate std;

    #[cfg(test)]
    pub(crate) use bevy::app::ScheduleRunnerPlugin;
    pub(crate) use bevy::app::{
        AppBuilder, CoreStage, Events, Plugin, PluginGroup, PluginGroupBuilder,
    };
    pub(crate) use bevy::asset::{AddAsset, Assets, Handle, HandleUntyped};
    #[cfg(test)]
    pub(crate) use bevy::asset::{AssetPlugin, HandleId};
    #[cfg(test)]
    pub(crate) use bevy::core::CorePlugin;
    #[cfg(test)]
    pub(crate) use bevy::ecs::system::CommandQueue;
    pub(crate) use bevy::ecs::{
        bundle::Bundle,
        entity::Entity,
        query::Changed,
        schedule::{ParallelSystemDescriptorCoercion, SystemLabel, SystemStage},
        system::{Commands, IntoSystem, Query, Res, ResMut},
    };
    pub(crate) use bevy::log::{error, info, warn};
    pub(crate) use bevy::math::{Vec2, Vec3};
    pub(crate) use bevy::reflect::{TypeUuid, Uuid};
    pub(crate) use bevy::render::{
        camera::Camera,
        color::Color,
        draw::{Draw, Visible},
        mesh::{Indices, Mesh},
        pipeline::{
            BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrite, CompareFunction,
            DepthBiasState, DepthStencilState, PipelineDescriptor, PrimitiveTopology,
            RenderPipeline, RenderPipelines, StencilFaceState, StencilState,
        },
        render_graph::base::MainPass,
        shader::{Shader, ShaderStage, ShaderStages},
        texture::TextureFormat,
    };
    pub(crate) use bevy::sprite::TextureAtlas;
    pub(crate) use bevy_tilemap_types::{
        dimension::{Dimension2, Dimension3, DimensionError},
        point::{Point2, Point3},
    };
    #[cfg(test)]
    pub(crate) use bevy::transform::components::Children;
    pub(crate) use bevy::transform::{
        components::{GlobalTransform, Parent, Transform},
        hierarchy::{BuildChildren, DespawnRecursiveExt},
    };
    pub(crate) use bevy::utils::{HashMap, HashSet};
    pub(crate) use bevy::window::WindowResized;

    pub(crate) use crate::bitflags::*;

    #[cfg(feature = "serde")]
    pub(crate) use serde::{Deserialize, Serialize};

    pub(crate) use std::{
        boxed::Box,
        clone::Clone,
        cmp::Ord,
        convert::{AsMut, AsRef, From, Into},
        default::Default,
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        iter::{Extend, IntoIterator, Iterator},
        ops::FnMut,
        option::Option::{self, *},
        result::Result::{self, *},
        vec::Vec,
    };

    // Macros
    pub(crate) use std::{vec, write};

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use bevy::log::debug;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use std::println;

    #[cfg(test)]
    pub(crate) use std::assert_eq;
}
