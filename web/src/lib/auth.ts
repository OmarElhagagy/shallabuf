import { cookies } from "next/headers";

export const getSessionToken = async () => {
  return (await cookies()).get("session")?.value;
};
