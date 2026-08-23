import { createSignal, onCleanup } from "solid-js";
import { onTooltipTrigger, type TooltipTrigger } from "./tooltipDelay";

/** The DOM half of the tooltip contract, shared by Tooltip and Popover
 *  (#591): one shown signal, one timer, one trigger funnel. tooltipDelay
 *  owns WHAT should happen and after how long; this owns making it happen —
 *  extracted so the machine cannot drift between the two surfaces that
 *  render it, which is exactly how the delay policy itself is kept single.
 *  `ignore` is evaluated per trigger, so a pointer-capability check stays
 *  fresh without a reactive subscription. */
export function createTooltipMachine(opts?: { ignore?: () => boolean }) {
  const [shown, setShown] = createSignal(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const clear = () => {
    if (timer !== undefined) {
      clearTimeout(timer);
      timer = undefined;
    }
  };
  // A trigger fired, then the control unmounted — without this the timer
  // still fires and sets a signal on a disposed component.
  onCleanup(clear);

  const hide = () => {
    clear();
    setShown(false);
  };

  const trigger = (kind: TooltipTrigger) => {
    if (opts?.ignore?.()) return;
    const action = onTooltipTrigger(kind);
    clear();
    if (action.kind === "delay") {
      timer = setTimeout(() => {
        setShown(true);
      }, action.ms);
    } else {
      setShown(action.kind === "show");
    }
  };

  return { shown, hide, trigger };
}
