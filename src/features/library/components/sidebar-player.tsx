import { IconMusic } from "@tabler/icons-react";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
} from "@/components/ui/card";

export default function SidebarPlayer() {
  return (
    <Card className="mx-auto w-fit gap-3">
      <CardContent className="flex flex-col justify-center gap-3">
        <div className="mx-auto flex aspect-square w-48 items-center justify-center rounded-lg bg-stone-400">
          <IconMusic size={120} className="text-stone-500" />
        </div>

        <div className="space-y-1 px-2">
          <p>Malefic Spell</p>
          <p className="text-muted-foreground text-xs">みや vs さか</p>
        </div>
      </CardContent>
    </Card>
  );
}
