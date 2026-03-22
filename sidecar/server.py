import grpc
from concurrent import futures
import splitter_pb2
import splitter_pb2_grpc
from wtpsplit import WtP

print("Loading wtpsplit model...")
wtp = WtP("wtp-canine-s-12l", ignore_legacy_warning=True)
print("Model ready")

class SplitterServicer(splitter_pb2_grpc.SplitterServicer):
    def Split(self, request, context):
        sentences = wtp.split(request.text,lang_code="en")
        return splitter_pb2.SplitResponse(sentences=sentences)

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=4))
    splitter_pb2_grpc.add_SplitterServicer_to_server(SplitterServicer(), server)
    server.add_insecure_port("[::]:50051")
    server.start()
    print("Sidecar listening on port 50051")
    server.wait_for_termination()

if __name__ == "__main__":
    serve()