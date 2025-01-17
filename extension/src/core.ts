import { FileServiceClient } from "./proto/LuteServiceClientPb";
import { IsFileStaleRequest, PutFileRequest } from "./proto/lute_pb";

const fileService = new FileServiceClient("http://localhost:22000");

export const putFile = async (fileName: string, content: string) => {
  const putFileRequest = new PutFileRequest();
  putFileRequest.setName(fileName);
  putFileRequest.setContent(content);

  await fileService.putFile(putFileRequest, {});
};
