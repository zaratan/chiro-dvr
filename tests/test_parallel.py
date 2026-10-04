from __future__ import annotations

from pathlib import Path

import pytest

from batdetect.detect import DetectConfig, Detection, detect_frames
from batdetect.parallel import Chunk, detect_video, exit_with_parent, plan_chunks
from batdetect.synthetic.injection import Injector
from batdetect.synthetic.trajectory import SyntheticBat
from batdetect.video import ColorFrame, open_video, read_gray_frames
from helpers import background, moving_square_frames, with_square, write_video

FIXTURE = Path(__file__).parent / "fixtures" / "video_092_original_3m24-4m05.mp4"
FAILING_FRAME = 450


class HookError(Exception):
    pass


class FailingHook:
    def __call__(self, frame: ColorFrame, frame_no: int) -> None:
        if frame_no == FAILING_FRAME:
            raise HookError(frame_no)


def plain_pass(video: Path, cfg: DetectConfig) -> dict[int, list[Detection]]:
    cap, info = open_video(video, cfg.work_width)
    try:
        return detect_frames(read_gray_frames(cap, info), info.fps, cfg, info.scale)
    finally:
        cap.release()


def square_video(path: Path) -> Path:
    frames = [with_square(background(160, 120, seed=i), 10 + (3 * i) % 130, 60) for i in range(900)]
    write_video(path, frames, fps=30)
    return path


def test_chunks_cover_every_frame_once_and_the_last_runs_to_the_end() -> None:
    chunks = plan_chunks(1000, workers=4, min_length=100)

    assert [c.start for c in chunks] == [0, 250, 500, 750]
    assert [c.stop for c in chunks] == [250, 500, 750, None]


def test_short_videos_are_not_split_into_tiny_chunks() -> None:
    assert plan_chunks(150, workers=8, min_length=100) == [Chunk(0, None)]


def test_split_detection_is_identical_to_a_single_pass(tmp_path: Path) -> None:
    video = square_video(tmp_path / "long.mp4")
    cfg = DetectConfig()
    cap, info = open_video(video, cfg.work_width)
    cap.release()

    single = detect_video(video, cfg, info, workers=1)[0]
    split = detect_video(video, cfg, info, workers=3)[0]

    assert split == single


def test_threaded_reading_matches_a_plain_pass_without_reader_thread(tmp_path: Path) -> None:
    video = square_video(tmp_path / "long.mp4")
    cfg = DetectConfig()
    cap, info = open_video(video, cfg.work_width)
    cap.release()

    assert detect_video(video, cfg, info, workers=1)[0] == plain_pass(video, cfg)


@pytest.mark.parametrize("workers", [1, 2])
def test_hook_error_reaches_the_caller_with_its_own_type(tmp_path: Path, workers: int) -> None:
    video = square_video(tmp_path / "long.mp4")
    cfg = DetectConfig()
    cap, info = open_video(video, cfg.work_width)
    cap.release()

    with pytest.raises(HookError):
        detect_video(video, cfg, info, workers, FailingHook)


def test_injected_targets_are_observed_on_every_frame_across_chunks(tmp_path: Path) -> None:
    video = tmp_path / "plain.mp4"
    write_video(video, moving_square_frames(900, step=(0, 0)), fps=30)
    cfg = DetectConfig()
    cap, info = open_video(video, cfg.work_width)
    cap.release()
    bat = SyntheticBat(0, 280, tuple((40.0 + k, 60.0) for k in range(60)), tuple(-60.0 for _ in range(60)), -60, 3)

    _, injectors = detect_video(video, cfg, info, 3, lambda: Injector([bat]))

    frames = {o.frame for inj in injectors for o in inj.observations.get(0, [])}
    assert frames == set(range(280, 340))


@pytest.mark.slow
def test_split_detection_on_the_real_clip_matches_a_single_pass() -> None:
    cfg = DetectConfig()
    cap, info = open_video(FIXTURE, cfg.work_width)
    cap.release()

    single = detect_video(FIXTURE, cfg, info, workers=1)[0]
    split = detect_video(FIXTURE, cfg, info, workers=4)[0]

    assert split == single
    assert single == plain_pass(FIXTURE, cfg)


def test_watching_the_parent_is_a_no_op_in_the_main_process() -> None:
    exit_with_parent()
