too-many-tracks = { $video }: { $count } tracks, above { $limit }: { $annotated ->
        [yes] clips and annotated video skipped
       *[no] clips skipped
    }
clips-redone = { $video }: { $names } written again with openh264 after a VideoToolbox error: { $causes }
video-step = [{ $position }/{ $total }] { $video } — { $step ->
        [detection] detection
        [tracking] tracking
        [summary] summary image
       *[clips] clips
    }
video-done = [{ $position }/{ $total }] { $video } — { $tracks ->
        [one] { $tracks } track
       *[other] { $tracks } tracks
    }
video-failed = [{ $position }/{ $total }] { $video } — { $too-many ->
        [yes] too many tracks
       *[no] failed
    }
failure = { $video }: { $cause ->
        [missing] file not found
        [read-refused] the system refuses to read this file
        [not-a-video] this file is not a readable video, or its copy is incomplete
        [unusable-name] FFmpeg cannot read the name of this file
        [damaged] the video is damaged or incomplete
        [unsupported] this video format is not handled
        [settings] the settings do not suit this video
        [disk-full] the disk is full
        [forbidden] writing is refused in { $place }
        [in-use] { $place } is open in another program
        [encoding] the clips could not be written
        [internal] internal error
       *[unexpected] unexpected error
    }
failure-todo = {"  "}what to do: { $cause ->
        [missing] check the path; the video may have been moved or renamed
        [read-refused] check its rights; on a Mac, allow the Terminal in System Settings, Privacy & Security, Files and Folders
        [not-a-video] if it comes from the binoculars, copy it again over the USB cable, not through Stream Vision 2; otherwise, move it out of the folder
        [unusable-name] rename the video with letters, digits and dashes
        [damaged] copy it again from the binoculars over the USB cable, not through Stream Vision 2, then run again; if it stays damaged, report it
        [unsupported] report it, with the detail below
        [settings] run again without --work-width or --osd-region
        [disk-full] free some space on the disk of { $folder }, then run again
        [forbidden] check the rights of this folder, or choose another one with -o
        [in-use] close it, then run again
        [encoding] run again; if it happens again, report it with the detail below
        [too-many-tracks] look at the summary image; if these are real passes, run again with a larger --max-tracks
       *[unexpected] report it with the detail below
    }
failure-detail = {"  "}detail: { $detail }
summary-part = part { $number } / { $count }
summary-tracks = { $count ->
        [one] { $count } track
       *[other] { $count } tracks
    }
summary-not-analysed = not analysed { $start } - { $end }
summary-more = + { $count } more
summary-unreadable = unreadable { $start } - { $end }
summary-more-unreadable = + { $count } more, { $duration } unreadable in all
band-not-analysed = not analysed
band-unreadable = unreadable
about = Detects and tracks bats in thermal binocular videos
inputs-help = Videos, or folders whose videos are all processed
out-dir-help = Folder that receives one folder of results per video
zoom-help = Slowed, magnified clip: auto for small or faint tracks only, all, or none
annotated-help = Also write the whole video with every track drawn
max-tracks-help = Above this many tracks, the video counts as failed and gets no clips or annotated video; 0 disables
clip-margin-help = Time kept before and after each track in its clip; seconds
osd-region-help = Hides a moving on-screen display from the detection, as fractions of the frame; repeat for several
help-help = Print help: -h the essential options, --help all of them
version-help = Print the version
no-region = none
beside-the-videos = batdetect/ beside each video
threshold-help = Smallest difference from the background that counts as a target; lower finds fainter targets and more noise; grey levels
min-hits-help = Tracks with fewer detections are dropped; lower keeps shorter tracks and more short flickers
max-blobs-help = Frames with at least this many spots more than six times the usual count are ignored, as when the binoculars move; 0 disables
work-width-help = Width the frames are reduced to for detection; wider sees smaller targets but takes longer; pixels
value-video = VIDEO
value-folder = FOLDER
value-which = WHICH
value-count = COUNT
value-seconds = SECONDS
value-level = LEVEL
value-pixels = PIXELS
usage-heading = Usage:
arguments-heading = Arguments
options-heading = Options
detection-heading = Settings that change the detection
default-value = [default:{"\u00A0"}{ $value }]
possible-values = [possible values: { $values }]
error-label = error:
tip-label = tip:
unexpected-argument = unexpected argument '{ $argument }' found
similar-argument = a similar argument exists: '{ $suggestion }'
similar-value = a similar value exists: '{ $suggestion }'
argument-as-value = to pass '{ $argument }' as a value, use '{ $escaped }'
invalid-value = invalid value '{ $value }' for '{ $argument }'
region-shape = --osd-region must be four fractions X0,Y0,X1,Y1 separated by commas, not { $value }
value-required = a value is required for '{ $argument }' but none was supplied
missing-arguments = the following required arguments were not provided:
used-twice = the argument '{ $argument }' cannot be used multiple times
unexpected-value = unexpected value '{ $value }' for '{ $argument }' found; no more were expected
invalid-utf8 = invalid UTF-8 was detected in one or more arguments
arguments-refused = the arguments are refused
decimal-point = write { $written }, with a point
more-information = For more information, try '{ $flag }'.
setting-refused = { $option } must be { $bound ->
        [positive] a finite number > 0
        [not-negative] a finite number ≥ 0
        [at-least] an integer ≥ { $limit }
       *[up-to] a finite number between 0 and { $limit }
    }, not { $value }
region-refused = --osd-region { $index } does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: { $region }
clip-margin-refused = --clip-margin must be a finite number of seconds, at least 0, not { $value }
settings-refused = the default settings are refused; please report it
option-refused = { $option } cannot be applied; please report it
inputs-refused = no video is processed:
input-refused = { $path }: { $reason ->
        [missing] not found
        [refused] reading refused
        [not-a-folder] part of the path is not a folder
       *[unavailable] unreachable
    }
input-without-video = { $path }: { $folder ->
        [yes] no video ({ $listed } or { $last }) in this folder
       *[no] not a video ({ $listed } or { $last })
    }
shared-folder = { $path }: results folder shared by { $videos }
no-video = no video ({ $listed } or { $last }) found in { $count ->
        [one] this folder
       *[other] these folders
    }:
folders-refused = no video is processed, the results cannot be written in { $count ->
        [one] this folder
       *[other] these folders
    }:
folder-refused = { $path }: { $access ->
        [refused] writing refused
        [not-a-folder] not a folder
        [missing] not found
        [read-only] read-only disk
        [disk-full] disk full
       *[unavailable] unreachable
    }
tally-title = Summary: { $total ->
        [one] { $total } video
       *[other] { $total } videos
    }
tally-done = { $count } done
tally-too-many-count = { $count } with too many tracks
tally-failed-count = { $count } failed
tally-video = video
tally-length = length
tally-tracks = tracks
tally-not-analysed = not analysed
tally-compute = compute
tally-failed-cell = failed
tally-too-many-cell = { $count }, too many
tally-too-many-heading = Too many tracks (table and summary image written, no clips):
tally-too-many = { $video }: { $count } tracks, above { $limit }
tally-failed-heading = { $failed ->
        [one] { $failed } video
       *[other] { $failed } videos
    } of { $total } failed:
tally-kept-heading = { $count ->
        [one] Table left as it is:
       *[other] Tables left as they are:
    }
tally-kept = { $video }: { $table } does not have the form of a tracks table{ $written ->
        [no] ; the new table is not written
       *[yes] {""}
    }
tally-changed = Changed settings: { $settings }
tally-regions = { $count ->
        [one] { $count } region
       *[other] { $count } regions
    }
tally-failure-todo = {"  "}what to do: { $cause ->
        [unsupported] report it, with the detail written during the run
        [encoding] run again; if it happens again, report it with the detail written during the run
       *[unexpected] report it with the detail written during the run
    }
common-uses = Common uses:
use-folder = Process a whole night folder, with the results in { $folder }/:
use-nights = Process several nights at once:
use-video = Process a single video:
use-out-dir = Write the results to another folder:
use-zoom = Make a slowed, magnified clip of every track:
use-annotated = Also write the whole video, with every track drawn:
example-night = night-oct-12
example-next-night = night-oct-13
example-results = results
all-options = All the options and the files produced:
files-heading = Files produced, in { $folder }/<video>/ beside the videos, or in <folder of -o>/<video>/:
file-video = <video>
file-summary = The summary image: every track of the video on one image
file-table = One row per track (start, end, length, speed, size), for a spreadsheet
file-clip = The clip of every track, named by its number and its start
file-zoom = The same, slowed and magnified, for small or faint tracks
file-annotated = The whole video with the tracks drawn, with --annotated
file-params = The settings used, to run the same computation again
files-long =
    A long or crowded video has several summary images, per 10 min slice ({ $period })
    or per part ({ $part }).
guide = Full guide:
inputs-skipped = { $count ->
        [one] one input is refused
       *[other] { $count } inputs are refused
    }:
skip-refused-question = Process { $count ->
        [one] the valid video
       *[other] the { $count } valid videos
    } anyway? (Y/n)
skip-refused-declined = no video is processed
skip-refused-help = Processes the valid videos without asking when some inputs are refused
tally-refused-count = { $count ->
        [one] { $count } input refused
       *[other] { $count } inputs refused
    }
tally-refused-heading = { $count ->
        [one] Refused input, not processed
       *[other] Refused inputs, not processed
    }:
interrupted = batch interrupted at video { $position }/{ $total }; running the same command again redoes the batch
