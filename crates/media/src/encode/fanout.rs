use std::fmt;
use std::path::Path;

use ffmpeg::picture;
use ffmpeg_next as ffmpeg;

use super::{Codec, Destination, Error, Opened, Settings, fill, receive, send};
use crate::frame::YuvFrame;

#[cfg(test)]
mod tests;

/// Encodes each frame once and writes it into every clip that holds it; each clip starts on a key frame.
pub struct Fanout {
    opened: Opened,
    next_pts: i64,
    key_next: bool,
    clips: Vec<Clip>,
    codec: Codec,
}

struct Clip {
    destination: Destination,
    first: i64,
    last: i64,
    written: i64,
}

impl fmt::Debug for Fanout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Fanout")
            .field("codec", &self.codec)
            .field("next_pts", &self.next_pts)
            .field("open_clips", &self.clips.len())
            .finish_non_exhaustive()
    }
}

impl Fanout {
    pub fn create(settings: &Settings) -> Result<Self, Error> {
        Ok(Self {
            opened: Opened::new(settings)?,
            next_pts: 0,
            key_next: false,
            clips: Vec::new(),
            codec: settings.codec,
        })
    }

    pub fn codec(&self) -> Codec {
        self.codec
    }

    pub fn open(&mut self, path: &Path, frames: usize) -> Result<(), Error> {
        let length = i64::try_from(frames).ok().filter(|length| *length > 0);
        let Some(length) = length else {
            return Err(Error::EmptyClip {
                path: path.to_path_buf(),
            });
        };
        let destination = Destination::create(path, &self.opened)?;
        self.clips.push(Clip {
            destination,
            first: self.next_pts,
            last: self.next_pts + length - 1,
            written: 0,
        });
        self.key_next = true;
        Ok(())
    }

    pub fn push(&mut self, frame: &YuvFrame) -> Result<(), Error> {
        fill(&mut self.opened.frame, frame)?;
        self.opened.frame.set_pts(Some(self.next_pts));
        let kind = if self.key_next {
            picture::Type::I
        } else {
            picture::Type::None
        };
        self.opened.frame.set_kind(kind);
        self.key_next = false;
        self.next_pts += 1;
        send(&mut self.opened.context, Some(&self.opened.frame))?;
        self.drain()
    }

    pub fn finish(mut self) -> Result<(), Error> {
        send(&mut self.opened.context, None)?;
        self.drain()?;
        match self.clips.iter().min_by_key(|clip| clip.first) {
            Some(clip) => Err(Error::ShortClip {
                path: clip.destination.path.clone(),
            }),
            None => Ok(()),
        }
    }

    fn drain(&mut self) -> Result<(), Error> {
        let time_base = self.opened.codec_time_base;
        while let Some(packet) = receive(&mut self.opened.context)? {
            let pts = packet.pts().ok_or(Error::UntimedPacket)?;
            let holding: Vec<usize> = (0..self.clips.len())
                .filter(|index| {
                    self.clips
                        .get(*index)
                        .is_some_and(|clip| (clip.first..=clip.last).contains(&pts))
                })
                .collect();
            if let Some((last, others)) = holding.split_last() {
                for index in others {
                    if let Some(clip) = self.clips.get_mut(*index) {
                        clip.take(packet.clone(), time_base)?;
                    }
                }
                if let Some(clip) = self.clips.get_mut(*last) {
                    clip.take(packet, time_base)?;
                }
            }
            for clip in self.clips.extract_if(.., |clip| clip.last <= pts) {
                clip.close()?;
            }
        }
        Ok(())
    }
}

impl Clip {
    fn take(&mut self, packet: ffmpeg::Packet, time_base: ffmpeg::Rational) -> Result<(), Error> {
        if self.written == 0 && !packet.is_key() {
            return Err(Error::NoKeyFrame {
                path: self.destination.path.clone(),
            });
        }
        self.written += 1;
        self.destination.write(packet, time_base, self.first)
    }

    fn close(self) -> Result<(), Error> {
        if self.written != self.last - self.first + 1 {
            return Err(Error::ShortClip {
                path: self.destination.path.clone(),
            });
        }
        self.destination.close()
    }
}
