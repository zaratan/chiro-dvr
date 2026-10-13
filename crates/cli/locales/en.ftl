processed = { $video }: { $tracks ->
        [one] { $tracks } track
       *[other] { $tracks } tracks
    } -> { $dest }
too-many-tracks = { $video }: { $count } tracks, above { $limit }: { $annotated ->
        [yes] clips and annotated video skipped
       *[no] clips skipped
    }
clips-redone = { $video }: { $names } written again with openh264 after a VideoToolbox error: { $causes }
failed = { $video }: failed: { $causes }
internal-error = { $video }: internal error: { $message }
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
inputs-help = videos, or folders whose videos are all processed
out-dir-help = folder that receives one folder of results per video
zoom-help = slowed, magnified clip: auto for small or faint tracks only, all, or none
annotated-help = also write the whole video with every track drawn
max-tracks-help = above this many tracks, the video counts as failed and gets no clips or annotated video; 0 disables
clip-margin-help = time kept before and after each track in its clip; seconds
osd-region-help = hides a moving on-screen display from the detection, as fractions of the frame; repeat for several
help-help = print help: -h the essential options, --help all of them
version-help = print the version
no-region = none
threshold-help = smallest difference from the background that counts as a target; lower finds fainter targets and more noise; grey levels
min-hits-help = tracks with fewer detections are dropped; lower keeps shorter tracks and more short flickers
max-blobs-help = frames with at least this many spots more than six times the usual count are ignored, as when the binoculars move; 0 disables
work-width-help = width the frames are reduced to for detection; wider sees smaller targets but takes longer; pixels
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
invalid-value-because = invalid value '{ $value }' for '{ $argument }': { $cause }
value-required = a value is required for '{ $argument }' but none was supplied
missing-arguments = the following required arguments were not provided:
used-twice = the argument '{ $argument }' cannot be used multiple times
unexpected-value = unexpected value '{ $value }' for '{ $argument }' found; no more were expected
invalid-utf8 = invalid UTF-8 was detected in one or more arguments
arguments-refused = the arguments are refused: { $cause }
more-information = For more information, try '{ $flag }'.
setting-refused = { $option } must be { $bound ->
        [positive] a finite number > 0
        [not-negative] a finite number ≥ 0
        [at-least] an integer ≥ { $limit }
       *[up-to] a finite number between 0 and { $limit }
    }, not { $value }
region-refused = --osd-region { $index } does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: { $region }
clip-margin-refused = --clip-margin must be a finite number of seconds, at least 0, not { $value }
settings-refused = the settings are refused: { $causes }
no-video = no video found
videos-refused = videos refused: { $causes }
