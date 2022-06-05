FROM node:18-alpine3.14

# install dependencies
WORKDIR /app
COPY package.json yarn.lock ./
RUN yarn

# Copy all local files into the image.
COPY . .

RUN yarn build

###
# Only copy over the Node pieces we need
# ~> Saves 35MB
###
FROM node:18-alpine3.14

WORKDIR /app
COPY --from=0 /app .
COPY . .

EXPOSE 3000
CMD ["node", "build"]