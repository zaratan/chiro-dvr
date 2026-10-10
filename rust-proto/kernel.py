"""Print OpenCV's Gaussian kernel for the default detection settings, as float32 bit patterns."""

import sys

import cv2
import numpy as np

from batdetect.detect import DetectConfig

width = int(sys.argv[1])
cfg = DetectConfig()
sigma = cfg.target_sigma / (width / min(cfg.work_width, width))
ksize = round(sigma * 4 * 2 + 1) | 1
kernel = np.asarray(cv2.getGaussianKernel(ksize, sigma, cv2.CV_32F), dtype=np.float32).ravel()
print(",".join(f"{v:08x}" for v in kernel.view(np.uint32)))
