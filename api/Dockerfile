FROM hsfzxjy/proxychains-ng:debian-bullseye-v4.15 AS pcng

FROM tiangolo/uvicorn-gunicorn-fastapi:python3.9-slim
ARG USE_PROXY
ENV USE_PROXY=${USE_PROXY}
COPY --from=pcng /opt/pcng /opt/pcng
RUN cat /opt/pcng/pcng
WORKDIR /app
RUN /opt/pcng/pcng apt-get update \
    && /opt/pcng/pcng apt-get install redis -y
COPY ./requirements.txt /app/
RUN bash -c "/opt/pcng/pcng pip install -r requirements.txt"
