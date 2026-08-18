// The shared primitives (#406) — one import path for both builds.
//
// A button exists once. If a surface needs a variant another does not have, the
// variant goes in the component here, not in a second button over there.
//
// The app keeps four of its own primitives that the design system also names —
// Card, Chevron, ControlGrid, Disclosure, PillToggle, Spinner and Tabs —
// because the system took its inventory FROM them. They move here when a
// second surface needs them, not before. ThemeToggle did: #395 put a theme
// control in the landing masthead, and #400 requires ONE mechanism rather
// than a second implementation, so it and lib/theme.ts came across together.

export { ArmedButton } from "./ArmedButton";
export {
  Button,
  type ButtonSize,
  type ButtonTone,
  type ButtonVariant,
} from "./Button";
export { Checkbox } from "./Checkbox";
export { Chip, type ChipTone, type ChipVariant } from "./Chip";
export { CountBubble, type CountBubbleTone } from "./CountBubble";
export { Dialog } from "./Dialog";
export { Field } from "./Field";
export { Icon } from "./Icon";
export { CONTROL_CLASS, Input } from "./Input";
export { Select } from "./Select";
export {
  StatusBadge,
  type StatusTone,
  type StatusVariant,
} from "./StatusBadge";
export { SummaryBanner, type BannerKind } from "./SummaryBanner";
export { Toast, ToastHost, retractToast, toast } from "./Toast";
export { ThemeToggle } from "./ThemeToggle";
export { Tooltip } from "./Tooltip";
export type { IconName } from "../icons/icons";
