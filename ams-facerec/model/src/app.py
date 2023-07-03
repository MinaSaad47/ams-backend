#!/usr/bin/env python3

import torch
from facenet_pytorch import MTCNN, InceptionResnetV1

from PIL import Image

from fastapi import FastAPI, UploadFile, File
import uvicorn

import os, argparse, shutil, io
from pathlib import Path
from typing import List

class Classifier():
    def __init__(self, model, preprocessing, classes, device):
        self.device = device
        self.model = model
        self.preprocessing = preprocessing
        self.classes = classes
    def classify(self, image) -> str:
        image_tensor = self.preprocessing(image)
        image_tensor = image_tensor.unsqueeze(0).to(self.device)
        with torch.no_grad():
            output = self.model.to(self.device)(image_tensor)
        idx = torch.argmax(output)
        return self.classes[idx]
    def to(self, device: torch.device):
        self.device = device
        return self

parent_exe_dir = Path(__file__).parent.absolute()

device: torch.device

mtcnn: MTCNN
resnet: InceptionResnetV1

classifier_path: str
classifier: Classifier

def initialize_variables():
    global mtcnn, resnet, classifier, classifier_path, device
    device_env = os.environ.get('COMPUTING_DEVICE', 'cpu')
    if device_env == 'cuda':
        if not torch.cuda.is_available():
            print('cuda not avaliable falling back to cpu')
            device = torch.device('cpu')
        else:
            device = torch.device('cuda')
    else:
        device = torch.device('cpu')
    print(f'using device: {device}')
    mtcnn = MTCNN(post_process=False).to(device).eval()
    print(f'loaded MTCNN')
    resnet = InceptionResnetV1(pretrained='vggface2').to(device).eval()
    print(f'loaded InceptionResnetV1')
    classifier_path = os.environ.get('CLASSIFIER_PATH', 'assets/classifier.pkl')
    try:
        classifier = torch.load(classifier_path)
        print(f'loaded Classifier')
    except:
        print(f'classifier not found')

app = FastAPI()

def embed_image(image_path) -> List[float]:
    global mtcnn, resnet
    image = Image.open(image_path).convert('RGB')
    image = mtcnn(image)
    embeddings = resnet(image.unsqueeze(0)).detach().cpu().numpy().squeeze()
    return embeddings.tolist()

def classify_image(image_path) -> str:
    global mtcnn, classifier
    image = Image.open(image_path).convert('RGB')
    image = mtcnn(image).to(torch.uint8).permute(1, 2, 0).detach().cpu().numpy()
    image = Image.fromarray(image)
    identity = classifier.classify(image)
    return identity


@app.post('/classify')
async def classifiy(image: UploadFile = File(...)) -> str:
    content = await image.read()
    return classify_image(io.BytesIO(content))

@app.post('/embed')
async def embbed(image: UploadFile) -> List[float]:
    return list(embed_image(io.BytesIO(await image.read())))

@app.post('/upload_classifier')
async def upload_classifier(model: UploadFile) -> str:
    global classifier, classifier_path, device
    with open(classifier_path, "wb") as buffer:
        shutil.copyfileobj(model.file, buffer)
    classifier = torch.load(classifier_path).to(device)

    return 'uploaded model successfully'

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Flask app with uvicorn server')
    parser.add_argument('--address', '-a', type=str, default='0.0.0.0',
                        help='Address to bind the server to')
    parser.add_argument('--port', '-p', type=int, default=5000,
                        help='Port to run the server on')
    args = parser.parse_args()
    initialize_variables()
    uvicorn.run(app, host=args.address, port=args.port)
