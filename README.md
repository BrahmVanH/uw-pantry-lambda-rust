# United Way Marquette Food Pantry Hub Rust Function
  
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)]

  
  ## Description 
  
  This lambda function supports the United Way of Marquette Food Pantry Hub. It offers user authentication and inventory updates for each food pantry.
  
  ## Table of Contents

  ⋆[Installation](#Installation)
  ⋆[Usage](#Usage)
  ⋆[Credits](#Credits)
  ⋆[License](#License)
  ⋆[Features](#Features)
  ⋆[Contributions](#Contributions)
  ⋆[Test](#Contributions)

  ## Installation 

  clone this repo, then from the root directory 
  <pre> $ cargo build </pre>
  remove .example suffix from .env file and fill in env's. This include the url to your dynamo db and a jwt secret. 

  ## Usage

  enter 
  <pre> $ cargo run </pre> 
  and enter "http://localhost:3000/graphql" in the url input in the API testing software of your choice. This function uses graphql, ensure your headers and request types are appropriate.

  ## Credits 

  brahm van houzen

  ## License

  (https://opensource.org/licenses/MIT)  

  ## Features

  - Basic CRUD
- User authentication
- wicked small memory footprint

  ## Technologies

  - Rust
- AWS Lambda
- DynamoDB
- GraphQL

  ## Contributions

  

  ## Test

  

  ## Questions

  If you have any questions about the project you can reach out to me via email or GitHub with the information below. 

  >Email: brahmvanh@gmail.com

  >GitHub: [brahmvanh](https://github.com/brahmvanh)
  