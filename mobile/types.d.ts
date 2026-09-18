/* eslint-disable */
declare module "*.svg" {
  import * as React from "react"
  import { SvgProps } from "react-native-svg"
  const content: React.FC<SvgProps>
  export default content
}

declare module "*.png" {
  const value: number
  export default value
}

// NativeWind side-effect CSS imports (e.g. `import "../global.css"`).
// The Expo-generated expo-env.d.ts normally provides this via expo/types,
// but it is gitignored — fresh checkouts and CI need it declared here.
declare module "*.css"
