# Intern Matchmaker

## Overview

Each year, J.B. Hunt accepts a large number of interns with different skillsets, backgrounds, and experience levels. All these interns need to be matched effectively to company projects. J.B. Hunt reports that a typical summer involves 50+ interns and over 25 projects, each with different requirements, expectations, deadlines, and priority levels. The company currently performs this matching process manually by human employees, which costs the company time and leaves the possibility for human error. To address these issues, they are exploring the possibility of automating that process and have tasked us with implementing a solution using AI. They hope to have a system that can quickly ingest information on projects and interns, accurately and effectively match interns to projects, considering the requirements of each project and the relevant experiences of each intern, and present these recommendations to the user in an interface that allows them to act on the algorithm’s output.

## About the API

The API exists to do the following things:

- Parse resumes into structured JSON
- Parse project spreadsheets and insert project information into the database
- Generate vector embeddings of the structured JSON and projects to be used by a custom neural network
- Handle requests to this neural net and relay the responses to the database
