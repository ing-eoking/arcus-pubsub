### (테스트 용) PUB/SUB 익스텐션 설치 및 실행 가이드


> [!WARNING] 이 PUB/SUB 익스텐션의 동시성과 성능은 처참하며, 에러가 많습니다.

```
make
```

#### 0. 준비 사항

- arcus-memcached

#### 1. 빌드

```
make
```

#### 2. arcus-memcached 실행

예를 들어, 엔진 파일이 /engines 디렉토리에 있고,
현재 프로젝트 경로가 /arcus-pubsub 이라면, 다음과 같이 실행할 수 있습니다:
```
memcached -E /engines/default_engine.so -X /arcus-pubsub/pubsub.so
```

#### 3. pub/sub 활용

- 채널 구독
```
subscribe iek
```

- 특정 채널에 미시지 발행
```
publish iek hello
```

- 응답 메시지는 다음과 같습니다.

```
CHANNEL iek 5
hello
END
```