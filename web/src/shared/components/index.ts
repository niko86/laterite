// The shared primitives (#406) — one import path for both builds.
//
// A button exists once. If a surface needs a variant another does not have, the
// variant goes in the component here, not in a second button over there.
//
// The app keeps four of its own primitives that the design system also names —
// Card, Chevron, ControlGrid, Disclosure, PillToggle, Spinner, Tabs and
// ThemeToggle — because the system took its inventory FROM them. They move here
// when a second surface needs them, not before.

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
export { Tooltip } from "./Tooltip";
export type { IconName } from "../icons/icons";
