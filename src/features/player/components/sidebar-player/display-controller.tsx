import { IconMusic } from "@tabler/icons-react";
import ControllerOverlay from "./controller-overlay";

export default function DisplayController() {
  return (
    <div className="relative mx-auto flex aspect-square w-48 items-center justify-center overflow-clip rounded-lg bg-stone-400">
      <ControllerOverlay />

      <IconMusic size={120} className="text-stone-500" />
    </div>
  );
}
