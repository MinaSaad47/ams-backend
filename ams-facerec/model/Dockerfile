FROM bitnami/pytorch:2.0.1

EXPOSE 5000

RUN pip install fastapi python-multipart uvicorn opencv-python facenet-pytorch --no-cache-dir

USER root
RUN useradd --uid 1000 appuser
RUN mkdir /.cache -p
RUN chown appuser:appuser /.cache
RUN mkdir /app -p
RUN chown appuser:appuser /app
USER appuser

RUN python3 -c "from facenet_pytorch import InceptionResnetV1; InceptionResnetV1(pretrained='vggface2')"

WORKDIR /app

COPY src/app.py /usr/bin/app.py

CMD [ "python3", "/usr/bin/app.py"]
