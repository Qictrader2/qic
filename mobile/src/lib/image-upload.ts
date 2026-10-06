/**
 * EXIF GPS stripping before photo upload.
 * Uses expo-image-manipulator to re-encode the image, which strips EXIF metadata.
 */
import * as ImageManipulator from "expo-image-manipulator"

export interface CapturedImage {
  uri: string
  name: string
  width: number
  height: number
  size?: number
}

const MAX_SIZE_BYTES = 2 * 1024 * 1024 // 2MB
const MAX_DIMENSION = 1920

/**
 * Strip EXIF (including GPS) and compress an image.
 * Re-encoding via ImageManipulator discards all EXIF metadata.
 */
export async function stripExifAndCompress(uri: string): Promise<CapturedImage> {
  // Resize if too large, compress to stay under 2MB
  const result = await ImageManipulator.manipulateAsync(
    uri,
    [{ resize: { width: MAX_DIMENSION } }],
    {
      compress: 0.8,
      format: ImageManipulator.SaveFormat.JPEG,
    }
  )

  return {
    uri: result.uri,
    name: `upload_${Date.now()}.jpg`,
    width: result.width,
    height: result.height,
  }
}

/**
 * Process multiple images: strip EXIF, compress, return ready-to-upload objects.
 */
export async function prepareImagesForUpload(
  uris: string[]
): Promise<CapturedImage[]> {
  return Promise.all(uris.map(stripExifAndCompress))
}
