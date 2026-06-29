FROM python:3.12-alpine
RUN apk add --no-cache iproute2 tmux
WORKDIR /app
COPY app.py .
CMD ["python", "app.py"]
