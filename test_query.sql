load 'build/debug/standarize_title.duckdb_extension';

create table job_titles (title text);

insert into job_titles (title) values
('software engineer'),
('product manager'),
('data scientist'),
('ux designer'),
('frontend developer'),
('backend developer'),
('devops engineer'),
('marketing manager'),
('hr specialist'),
('financial analyst'),
('content creator'),
('sales executive'),
('business analyst'),
('it consultant'),
('project manager'),
('quality assurance engineer'),
('graphic designer'),
('customer relations manager'),
('supply chain analyst'),
('digital strategist');

select title, standarize_title(title) from job_titles;