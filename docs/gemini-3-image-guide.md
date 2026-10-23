# Gemini 3 Pro Image Model Calling Guide

This document details how to use Google's `gemini-3-pro-image` (Imagen 3) model in the **Antigravity** project. This project has encapsulated the model with full compatibility with the OpenAI protocol, and extended support for the native photography aspect ratio, character generation security policy, and **Image-to-Image** functionality.

## 1. Basic information

* **Model ID**: `gemini-3-pro-image` (supports alias `gemini-3-pro-image-preview`)
* **Interface path**:
    * `/v1/images/generations` (Text-to-Image)
    * `/v1/images/edits` (Image-to-Image / edit)
    * `/v1/chat/completions` (compatibility mode)
* **Underlying model**: Google Imagen 3 (Gemini Native)

---

## 2. Text-to-Image

Call `/v1/images/generations`, supporting the following parameters:

### 2.1 Size / Aspect Ratio

The `size` parameter supports two input formats, and the system will automatically parse and map to the standard scale supported by Gemini:

1. **Directly enter the ratio (recommended)**: such as `"16:9"`, `"4:3"`, `"1:1"`. This way is the most intuitive and 100% accurate mapping.
2. **Input resolution (compatible)**: such as `"1920x1080"`, `"1024x1024"`. The system automatically calculates its aspect ratio (e.g. 1920/1080 ≈ 1.77) and normalizes it to the nearest standard ratio (16:9).

**⚠️ IMPORTANT NOTE**: Gemini (Imagen 3) **does not support customizing arbitrary pixel sizes**.
Whether you enter `"1920x1080"` or `"16:9"` in `size`, the resulting **actual physical resolution** is determined by only the following two factors:
1. **Aspect Ratio** (parsed by `size`)
2. **Image quality level** (determined by `quality` parameter: `1k`/`2k`/`4k`)

*Example: Entering `size: "1920x1080"` (16:9) and `quality: "standard"` (1k), the actual generated image size is **1376x768** (1K resolution at 16:9), not 1920x1080. *

| target ratio | Applicable scenarios | `size` parameter example (resolution) | Remark |
| :--- | :--- | :--- | :--- |
| **16:9** | Widescreen, cinematic | `1920x1080`, `1280x720` | Standard widescreen |
| **9:16** | Mobile wallpapers, Stories | `1080x1920`, `720x1280` | vertical full screen |
| **1:1** | Avatar, Instagram | `1024x1024` | Default scale |
| **4:3** | Traditional photography, monitor | `1024x768`, `800x600` | |
| **3:4** | portrait photography | `768x1024`, `600x800` | |
| **21:9** | Ultrawidescreen, movies | `2560x1080` | movie screen |
| **3:2** | **[New]** Full frame DSLR | `1500x1000` | Classic photography proportions |
| **2:3** | **[New]** Vertical composition photography | `1000x1500` | Posters, vertical paintings |
| **5:4** | **[New]** Large format | `1280x1024` | art photography |
| **4:5** | **[NEW]** Social media vertical image | `1024x1280` | Ins best display ratio |

> **Tip**: You don't need to match the pixel values ​​exactly, just an aspect ratio close to the ratio above (tolerance 0.05) for automatic recognition.

### 2.2 Image quality and resolution (Quality)

Control the granularity of the generation via the `quality` parameter.

| Parameter value (`quality`) | Corresponding Gemini settings | illustrate |
| :--- | :--- | :--- |
| `standard` / `1k` | Image Size: `1K` | Fast generation, suitable for quick verification (default) |
| `medium` / `2k` | Image Size: `2K` | Balancing quality and speed |
| `hd` / `4k` | Image Size: `4K` | **Extremely high quality**, richest in details, takes slightly longer |

#### Resolution comparison table (Gemini 3 Pro Image)

| aspect ratio | 1K resolution (Standard) | 2K resolution (Medium) | 4K resolution (HD) |
| :--- | :--- | :--- | :--- |
| **1:1** | 1024x1024 | 2048x2048 | 4096x4096 |
| **2:3** | 848x1264 | 1696x2528 | 3392x5056 |
| **3:2** | 1264x848 | 2528x1696 | 5056x3392 |
| **3:4** | 896x1200 | 1792x2400 | 3584x4800 |
| **4:3** | 1200x896 | 2400x1792 | 4800x3584 |
| **4:5** | 928x1152 | 1856x2304 | 3712x4608 |
| **5:4** | 1152x928 | 2304x1856 | 4608x3712 |
| **9:16** | 768x1376 | 1536x2752 | 3072x5504 |
| **16:9** | 1376x768 | 2752x1536 | 5504x3072 |
| **21:9** | 1584x672 | 3168x1344 | 6336x2688 |

### Call example (Python)

```python
import requests

url = "http://localhost:8045/v1/images/generations"
headers = {
    "Content-Type": "application/json",
    "Authorization": "Bearer <token>"
}
data = {
    "model": "gemini-3-pro-image",
    "prompt": "A futuristic city with flying cars, cinematic lighting, 8k",
    "size": "16:9",
    "quality": "hd",
    "n": 1
}

response = requests.post(url, headers=headers, json=data)
print(response.json())
```

## 3. Image-to-Image / Edits 🔥 [New]

Call the `/v1/images/edits` interface to support generation through reference images.

*   **Content-Type**: `multipart/form-data`
* **Support multiple pictures**: Multiple reference pictures can be uploaded at the same time.

### Form field descriptions

| Field name | type | Required | illustrate |
| :--- | :--- | :--- | :--- |
| `prompt` | String | yes | text prompt word |
| `image1`...`imageN` | File | yes | **Reference Picture File**. Supports `image1`, `image2` and other file fields with any names (not standard `image` or `mask`). |
| `image` | File | no | (Compatible with OpenAI standards) Main image |
| `mask` | File | no | (Compatible with OpenAI standards) Mask image |
| `aspect_ratio` | String | no | Explicitly specify the ratio, such as `"16:9"` (has higher priority than `size`) |
| `image_size` | String | no | Explicitly specify the resolution, such as `"2K"`, `"4K"` (higher than `quality`) |
| `style` | String | no | Style description will be automatically appended to Prompt |
| `n` | Integer | no | Generate quantity (default 1) |
| `model` | String | no | Model name (default `gemini-3.1-flash-image`) |

### Call example (Python)

```python
import requests

url = "http://localhost:8045/v1/images/edits"
headers = {
    "Authorization": "Bearer <token>"
}
# Support multiple reference images (image1, image2, ...)
files = {
    "image1": open("/path/to/reference_1.jpg", "rb"),
    "image2": open("/path/to/reference_2.jpg", "rb")
}
data = {
    "prompt": "A cyberpunk city street based on this layout",
    "aspect_ratio": "16:9",
    "image_size": "4K",
    "style": "watercolor"
}

response = requests.post(url, headers=headers, files=files, data=data)
print(response.json())
```

---

## 4. Magic Suffix

In addition to standard JSON parameters, this project also supports directly specifying parameters in the **model name** (convenient for use in clients that do not support custom parameters).

**Format**: `gemini-3-pro-image-{ratio}-{quality}`

* **Scale suffix**: `-16x9`, `-9x16`, `-4x3`, `-3x4`, `-3x2`, `-2x3`, etc.
* **Image quality suffix**: `-4k` (corresponding to hd), `-2k` (corresponding to medium).

**Example**:
Using the model name `gemini-3-pro-image-16x9-4k` is equivalent to:
*   `size`: "1920x1080" (16:9)
*   `quality`: "hd"

> **Note**: If `size` or `quality` is explicitly passed in the JSON Body, the priority of the parameters in the Body is **higher** than the model name suffix.

---

## 5. FAQ

1. **Q: Why do I set `size: "1234x5678"` but the proportion of the generated image is wrong? **
    * **A**: The system will normalize the dimensions you enter to the 10 standard ratios supported by Gemini (see Section 2.1). If your ratio is very specific and doesn't match any standard ratio (tolerance > 0.05), the system will fall back to the default **1:1**. It is recommended to use the resolution in the example directly.

2. **Q: Does it support generating multiple pictures at one time? **
    * **A**: Supported. Although Gemini upstream is limited to generating 1 piece per request, the Antigravity proxy layer will automatically process the `n` parameter concurrently. For example, if `n: 4` is set, the system will initiate 4 requests in parallel and return the combined results.

3. **Q: `person_generation` parameter reports an error? **
    * **A**: Please ensure that the parameter is located at the **root level** of JSON (same level as `prompt`, `model`) and not nested in other fields. Supports `snake_case` (`person_generation`) and `camelCase` (`personGeneration`).
