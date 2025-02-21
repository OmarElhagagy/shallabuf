import { useDroppable } from "@dnd-kit/core";

export const Dropzone = (props: React.HTMLAttributes<HTMLDivElement>) => {
  const { setNodeRef } = useDroppable({
    id: "dropzone",
  });

  return (
    <div {...props} ref={setNodeRef}>
      {props.children}
    </div>
  );
};
