# Add video with mongodbsh

connection string: "mongodb://localhost:4000/video-streaming"

add dataset: db.videos.insertOne({"_id" : new ObjectId("5d9e690ad76fe06a3d7ae416"),"videoPath" : "SampleVideo_1280x720_1mb.mp4"})
