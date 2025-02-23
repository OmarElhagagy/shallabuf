"use client";

import { logoutAction } from "~/actions/logout";
import { Button } from "~/components/ui/button";

export const LogoutButton = () => {
  return (
    <Button variant="outline" onClick={logoutAction}>
      Logout
    </Button>
  );
};
