pub mod arduino_strip;
pub mod led_color;
pub mod strip;

use anyhow::Result;
use log::trace;
use tokio::sync::mpsc;

use crate::core::led_color::LedColor;
use crate::modes::behaviors::{Behavior, BehaviorMod};
use crate::modes::sources::{Source, SourceMod};
use strip::Strip;

pub async fn poll(
    strip: Box<dyn Strip>,
    source_mod: SourceMod,
    behavior_mod: BehaviorMod,
) -> Result<()> {
    let mut source = source_mod.get_source().await?;
    let mut behavior = behavior_mod.get_behavior(strip).await?;

    let (tx, mut rx) = mpsc::channel::<Vec<LedColor>>(1);

    let source_task = async move {
        while let Ok(colors) = source.poll_next().await {
            if tx.send(colors).await.is_err() {
                break;
            }
            trace!("Source task iteration ended!");
        }
    };

    let behavior_task = async move {
        while let Some(colors) = rx.recv().await {
            let _ = behavior.poll_next(&colors).await;
            trace!("Behavior task iteration ended!");
        }
    };

    tokio::join!(source_task, behavior_task);

    Ok(())
}
